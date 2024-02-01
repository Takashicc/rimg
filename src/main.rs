mod constants;
mod executor;
mod params;
mod validation;
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
    Rename(params::rename::RenameParams),
    /// Compress files in each directory
    Compress(params::compress::CompressParams),
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Rename(v)) => {
            executor::rename::execute(v);
        }
        Some(Commands::Compress(v)) => {
            executor::compress::execute(v);
        }
        None => eprintln!("No subcommand provided!\nCheck the subcommands with `rimg -h`"),
    }
}
