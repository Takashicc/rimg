use crate::constants::file::RAR_EXTENSION;
use crate::validation;
use clap::Parser;

#[derive(Parser)]
/// Params for compress subcommand
pub struct CompressParams {
    #[arg(
        value_parser = validation::filepath::dir_exists,
        help = "Input directory"
    )]
    pub input_dir: String,

    #[arg(
        short,
        long,
        value_parser = validation::filepath::dir_exists,
        help = "Output directory"
    )]
    pub output_dir: Option<String>,

    #[arg(
        short,
        long,
        default_value_t = String::from(RAR_EXTENSION),
        value_parser = validation::filepath::format_type_check,
        help = "Compress file format type"
    )]
    pub format_type: String,

    #[arg(
        short,
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
