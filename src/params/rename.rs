use crate::validator;
use clap::{value_parser, Parser};

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
        value_parser = value_parser!(u8).range(1..=6),
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
        value_parser = value_parser!(u32).range(0..),
        help = "Initial number"
    )]
    pub initial: u32,

    #[arg(
        short,
        long,
        default_value_t = 1,
        value_parser = value_parser!(u8).range(1..),
        help = "Number of steps to count each files"
    )]
    pub step: u8,

    #[arg(short, long, help = "Execute immediately or not")]
    pub yes: bool,
}
