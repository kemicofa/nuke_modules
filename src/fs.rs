use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, bail};
use async_recursion::async_recursion;
use tokio::{sync::Semaphore, task::JoinSet};
use tracing::{debug, warn};

use crate::node_modules::NodeModules;

pub async fn calc_node_modules_sizes(
    node_modules: &mut Vec<NodeModules>,
    max_concurrency: usize,
) -> anyhow::Result<u64> {
    let sem = Arc::new(Semaphore::new(max_concurrency));
    let mut set: JoinSet<(usize, u64)> = JoinSet::new();

    for (i, node_module) in node_modules.iter().enumerate() {
        let path = node_module.path.clone();
        let sem_child = sem.clone();
        set.spawn(async move {
            let size = calc_dir_size(path, sem_child).await.unwrap_or(0);
            (i, size)
        });
    }

    let mut total_size_bytes: u64 = 0;
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((i, size)) => {
                total_size_bytes += size;
                node_modules[i].size = Some(size);
            }
            Err(e) => warn!("Join error in child task: {e}"),
        }
    }

    anyhow::Ok(total_size_bytes)
}

#[async_recursion]
async fn calc_dir_size(start_path: PathBuf, sem: Arc<Semaphore>) -> anyhow::Result<u64> {
    let mut set: JoinSet<anyhow::Result<u64>> = JoinSet::new();
    let mut size: u64 = 0;

    // When permit and start_dir go out of scope, they are auto dropped
    {
        let _permit = sem.clone().acquire_owned().await.with_context(|| {
            format!("Failed to acquire semaphore when searching for node_modules")
        })?;

        let mut start_dir = tokio::fs::read_dir(start_path).await.with_context(|| {
            format!("Failed to read directory when attempting to calculate size")
        })?;
        loop {
            let dir_entry = match start_dir.next_entry().await {
                Ok(Some(dir_entry)) => dir_entry,
                Ok(None) => {
                    // No more files to read in directory
                    break;
                }
                Err(e) => {
                    warn!("Error reading directory entry: {}", e);
                    continue;
                }
            };

            // Ignore errors, set to default if can't determine size
            size += dir_entry
                .metadata()
                .await
                .map_or(0, |metadata| metadata.len());

            let file_type = match dir_entry.file_type().await {
                Ok(file_type) => file_type,
                Err(e) => {
                    warn!(
                        "Skipping; Failed to read file type of directory entry: {}",
                        e
                    );
                    continue;
                }
            };

            // Skip anything that is not a directory
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }

            let path = dir_entry.path();
            let sem_child = sem.clone();
            set.spawn(async move { calc_dir_size(path, sem_child).await });
        }
    }

    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok(s)) => {
                size += s;
            }
            Ok(Err(e)) => warn!("Child calc size failed: {e}"),
            Err(e) => warn!("Join error in child task: {e}"),
        }
    }

    anyhow::Ok(size)
}

pub async fn nuke_node_modules(
    node_modules: Vec<NodeModules>,
    max_concurrency: usize,
) -> anyhow::Result<u64> {
    let mut set: JoinSet<anyhow::Result<u64>> = JoinSet::new();
    let mut node_modules_iter = node_modules.iter();
    let sem = Arc::new(Semaphore::new(max_concurrency));

    while let Some(node_module) = node_modules_iter.next() {
        let path = node_module.path.clone();
        let bytes_to_delete = node_module.size.unwrap_or(0);
        let sem_child = sem.clone();
        set.spawn(async move {
            let _permit = sem_child
                .acquire_owned()
                .await
                .with_context(|| format!("Failed to acquire semaphore when nuking node_modules"))?;
            match tokio::fs::remove_dir_all(path).await {
                Ok(()) => anyhow::Ok(bytes_to_delete),
                Err(e) => bail!("Failed to remove node_modules: {}", e),
            }
        });
    }

    let mut total_bytes_deleted: u64 = 0;

    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok(bytes_deleted)) => {
                total_bytes_deleted += bytes_deleted;
            }
            Ok(Err(e)) => warn!("{e}"),
            Err(e) => warn!("Join error in child task: {e}"),
        }
    }

    anyhow::Ok(total_bytes_deleted)
}

pub async fn find_node_modules(
    start_path: PathBuf,
    max_concurrency: usize,
) -> anyhow::Result<Vec<NodeModules>> {
    let sem = Arc::new(Semaphore::new(max_concurrency));

    let node_modules = find_node_modules_inner(start_path, sem).await;

    node_modules
}

pub const NODE_MODULES: &str = "node_modules";

#[async_recursion]
async fn find_node_modules_inner(
    start_path: PathBuf,
    sem: Arc<Semaphore>,
) -> anyhow::Result<Vec<NodeModules>> {
    let mut node_modules: Vec<NodeModules> = Vec::new();
    let mut set: JoinSet<anyhow::Result<Vec<NodeModules>>> = JoinSet::new();

    // Scope so that permit and start_dir are auto dropped
    {
        // Wait till there is availability to start processing directory
        let _permit = sem.clone().acquire_owned().await.with_context(|| {
            format!("Failed to acquire semaphore when searching for node_modules")
        })?;

        debug!("Number of available permits: {}", sem.available_permits());

        let mut start_dir = tokio::fs::read_dir(start_path.clone())
            .await
            .with_context(|| format!("Failed to read directory {}", start_path.display()))?;

        loop {
            let dir_entry = match start_dir.next_entry().await {
                Ok(Some(dir_entry)) => dir_entry,
                Ok(None) => {
                    // No more files to read in directory
                    break;
                }
                Err(e) => {
                    warn!("Error reading directory entry: {}", e);
                    continue;
                }
            };

            let file_type = match dir_entry.file_type().await {
                Ok(file_type) => file_type,
                Err(e) => {
                    warn!(
                        "Skipping; Failed to read file type of directory entry: {}",
                        e
                    );
                    continue;
                }
            };

            // Skip anything that is not a directory
            if !file_type.is_dir() || file_type.is_symlink() {
                continue;
            }

            let file_name = dir_entry.file_name();

            if file_name == NODE_MODULES {
                debug!(
                    "Found node_modules directory: {}",
                    dir_entry.path().display()
                );
                node_modules.push(NodeModules::new(dir_entry.path()));
                continue;
            }

            // A directory that is not a node_modules folder
            let path = dir_entry.path();
            let sem_child = sem.clone();
            set.spawn(async move { find_node_modules_inner(path, sem_child).await });
        }
    }

    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(Ok(mut v)) => node_modules.append(&mut v),
            Ok(Err(e)) => warn!("Child search failed: {e}"),
            Err(e) => warn!("Join error in child task: {e}"),
        }
    }

    Ok(node_modules)
}
