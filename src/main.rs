mod executor;
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
    Rename(RenameParams),
}

#[derive(Parser)]
pub struct RenameParams {
    #[arg(
        value_parser = validator::dir_exists,
        help = "Target directory"
    )]
    input_dir: String,

    #[arg(
        short,
        long,
        default_value_t = 4,
        value_parser = validator::is_positive_number,
        help = "Number of digits for renaming"
    )]
    digit: u8,

    #[arg(
        short,
        long,
        default_value_t = String::from("jpg"),
        help = "Target file extension"
    )]
    extension: String,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = validator::start_from_zero,
        help = "Initial number"
    )]
    initial: u8,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = validator::is_positive_number,
        help = "Number of steps to count each files"
    )]
    step: u8,

    #[arg(short, long, help = "Execute immediately or not")]
    yes: bool,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Rename(v)) => {
            executor::rename(v);
        }
        None => panic!("No sub command provided!"),
    }
}
