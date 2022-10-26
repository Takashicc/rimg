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
    pub initial: u8,

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

#[derive(Parser)]
pub struct CompressParams {
    #[arg(
        value_parser = validator::dir_exists,
        help = "Input directory"
    )]
    pub input_dir: String,

    #[arg(
        short,
        long,
        value_parser = validator::dir_exists,
        help = "Output directory"
    )]
    pub output_dir: Option<String>,

    #[arg(
        short,
        long,
        default_value_t = String::from("rar"),
        value_parser = validator::format_type_check,
        help = "Compress file format type"
    )]
    pub format_type: String,

    #[arg(
        long,
        conflicts_with = "validate_only",
        help = "Check the compressed file is not corrupted after the file was created"
    )]
    pub validate: bool,

    #[arg(
        long,
        conflicts_with = "validate",
        help = "Just check the compressed file is not corrupted"
    )]
    pub validate_only: bool,

    #[arg(short, long, help = "Execute immediately or not")]
    pub yes: bool,
}
