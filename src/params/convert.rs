use crate::validator;
use clap::{value_parser, Parser};

#[derive(Parser)]
/// Params for convert subcommand
pub struct ConvertParams {
    #[arg(
        value_parser = validator::dir_exists,
        help = "Target directory"
    )]
    pub input_dir: String,

    #[arg(
        // TODO value_parser
        help = "File type to be converted to"
    )]
    pub convert_to: String,

    #[arg(short, long, help = "Execute immediately or not")]
    pub yes: bool,
}
