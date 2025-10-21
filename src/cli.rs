use clap::{Parser, arg};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Auto respond "yes" to delete node_modules
    #[arg(short, long, default_value_t = false)]
    pub yes: bool,
}
