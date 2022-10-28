use crate::validator;
use clap::Parser;

#[derive(Parser)]
pub struct RenameParams {
    #[arg(
        value_parser = validator::dir_exists,
        help = "Target directory"
    )]
    pub input_dir: String,

    #[arg(
        short,
        long,
        default_value_t = 4,
        value_parser = validator::is_positive_number,
        help = "Number of digits for renaming"
    )]
    pub digit: u8,

    #[arg(
        short,
        long,
        default_values = vec!["jpg", "jpeg"],
        value_parser = validator::extension_check,
        help = "Target file extension"
    )]
    pub extensions: Vec<String>,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = validator::start_from_zero,
        help = "Initial number"
    )]
    pub initial: u32,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = validator::is_positive_number,
        help = "Number of steps to count each files"
    )]
    pub step: u8,

    #[arg(short, long, help = "Execute immediately or not")]
    pub yes: bool,
}
