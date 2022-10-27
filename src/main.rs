mod constant;
mod executor;
mod params;
mod validator;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Rename files in each directory to sequential number
    Rename(params::RenameParams),
    /// Compress files in each directory
    Compress(params::CompressParams),
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Rename(v)) => {
            executor::rename(v);
        }
        Some(Commands::Compress(v)) => {
            executor::compress(v);
        }
        None => panic!("No sub command provided!"),
    }
}
