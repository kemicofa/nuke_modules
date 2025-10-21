use std::{num::NonZero, thread};

pub const DEFAULT_NB_THREADS: usize = 4;

/// Function that determines how many threads to use.
/// Essentially 1 core = 1 thread. If unable to determine
/// how many cores are on the system, it defaults to 4 threads.
pub fn get_nb_threads_to_spawn() -> NonZero<usize> {
    thread::available_parallelism().unwrap_or(NonZero::new(DEFAULT_NB_THREADS).unwrap())
}
