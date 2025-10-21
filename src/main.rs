//! Command line utility that cleans all node_modules from the current
//! working directory.
//!
//! ```sh
//! # install globally
//! cargo install nuke_modules
//! ```
//!
//! Note: confirmation is requested before proceeding with the deleting.
//!
//! Anyone working on NodeJs projects, will quickly realize how much
//! space node_modules takes up on the system. Especially when working
//! in a micro-service architecture or having multiple projects.
//!
//! Not all projects need to be worked on so cleaning the node_modules
//! from those projects will liberate much room.

use std::env::current_dir;

use ::tracing::debug;
use anyhow::{Context, bail};
use clap::Parser;
use inquire::Confirm;
use tokio::runtime::Builder;

use crate::{
    bytes::bytes_to_human_readable,
    cli::Cli,
    fs::{calc_node_modules_sizes, find_node_modules, nuke_node_modules},
    node_modules::NodeModules,
    threads::get_nb_threads_to_spawn,
    tracing::init_tracing,
};

mod bytes;
mod cli;
mod fs;
mod node_modules;
mod threads;
mod tracing;

/// Every OS has a limit on how many files can be open at once.
/// On Unix like systems, this can be checked with `ulimit -n`.
/// We'll assume that most systems can handle having 512 open files at once.
const MAX_CONCURRENCY: usize = 512;

fn main() -> anyhow::Result<()> {
    init_tracing();

    let cli = Cli::parse();

    let nb_threads_to_spawn = get_nb_threads_to_spawn();

    debug!(
        "Available parallelism (logical cores): {:?}",
        nb_threads_to_spawn
    );

    let rt = Builder::new_multi_thread()
        .worker_threads(nb_threads_to_spawn.into())
        .enable_all() // enable I/O, time, etc.
        .build()
        .with_context(|| format!("Failed to build multi thread runtime"))?;

    let cwd = current_dir().with_context(|| format!("Failed to get current working directory"))?;

    let mut node_modules: Vec<NodeModules> =
        rt.block_on(async { find_node_modules(cwd, MAX_CONCURRENCY).await })?;

    let node_modules_count = node_modules.len();

    if node_modules_count == 0 {
        println!("📦 No node_modules were found.");
        return Ok(());
    }

    let total_byte_size: u64 = rt
        .block_on(async { calc_node_modules_sizes(&mut node_modules, MAX_CONCURRENCY).await })
        .unwrap_or(0);

    // sort by ascending bytes
    node_modules.sort_by(|a, b| a.size.cmp(&b.size));

    for (index, node_module) in node_modules.iter().enumerate() {
        println!("{}. {node_module}", index + 1);
    }

    println!(
        "📦 Found {node_modules_count} node_modules ({})",
        bytes_to_human_readable(total_byte_size)
    );

    let answer = if cli.yes {
        Ok(true)
    } else {
        Confirm::new("💥 Nuke these node_modules?")
            .with_default(false)
            .prompt()
    };

    match answer {
        Ok(true) => {
            let total_bytes_deleted =
                rt.block_on(async { nuke_node_modules(node_modules, MAX_CONCURRENCY).await })?;

            println!(
                "✅ deleted {} worth of node_modules!",
                bytes_to_human_readable(total_bytes_deleted)
            );
        }
        Ok(false) => {
            println!("🥲 That's too bad, I really wanted to nuke'em.");
        }
        Err(_) => bail!("Error with questionnaire, try again later."),
    }

    Ok(())
}
