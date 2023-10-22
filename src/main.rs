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
    Rename(params::rename::RenameParams),
    /// Compress files in each directory
    Compress(params::compress::CompressParams),
    /// TODO
    Convert(params::convert::ConvertParams),
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
        Some(Commands::Convert(v)) => {
            executor::convert::execute(v);
        }
        None => eprintln!("No subcommand provided!\nCheck the subcommands with `rimg -h`"),
    }
}
