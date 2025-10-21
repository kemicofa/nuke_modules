use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("error"));

    // Build the subscriber
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter) // reads RUST_LOG
        .finish();

    // Make it the default subscriber
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");
}
