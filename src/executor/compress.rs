use crate::constant::RAR_PATH;
use crate::executor::utils::{ask, get_progress_bar, have_extension, is_dir, is_hidden, is_parent};
use crate::params::compress::CompressParams;
use colored::Colorize;
use execute::Execute;
use std::collections::HashMap;
use std::path::Path;
use std::process::{self, Command};
use walkdir::{DirEntry, WalkDir};

/// Compress each directory and validate
///
/// # Arguments
///
/// * `params` - Compress params
pub fn execute(params: &CompressParams) {
    // Check rar executable
    if params.format_type == RAR_PATH
        && Command::new(RAR_PATH)
            .execute_check_exit_status_code(0)
            .is_err()
    {
        eprintln!("{}", "rar executable not found!.".red().bold());
        eprintln!("Abort...");
        process::exit(1);
    }

    if params.validate_only {
        let files = WalkDir::new(&params.input_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|v| have_extension(&params.format_type, v.path()))
            .map(|v| (v.file_name().to_string_lossy().to_string(), false))
            .collect::<HashMap<String, bool>>();

        if files.is_empty() {
            eprintln!(
                "{}",
                format!(
                    "There are no {} files to be executed",
                    params.format_type.to_uppercase()
                )
                .red()
                .bold()
            );
            eprintln!("Abort...");
            process::exit(0);
        }

        println!("{} files will be executed", files.len());

        ask(params.yes);

        validate_files(&params.input_dir, files);
        process::exit(0);
    }

    let directories = WalkDir::new(&params.input_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|v| is_dir(v) && !is_hidden(v) && !is_parent(v.path(), &params.input_dir))
        .collect::<Vec<DirEntry>>();

    if directories.is_empty() {
        eprintln!("{}", "There are no directories to be executed".red().bold());
        println!("Abort...");
        process::exit(0);
    }

    let execute_target_len = directories.len();
    println!("{} directories will be executed", execute_target_len);

    ask(params.yes);

    let bar = get_progress_bar(execute_target_len as u64);

    let mut compress_success_files = HashMap::<String, bool>::new();
    let mut compress_error_files = Vec::<String>::new();
    for directory in &directories {
        let output_filepath = if let Some(v) = &params.output_dir {
            Path::new(&v).join(format!("{}.rar", directory.file_name().to_string_lossy()))
        } else {
            Path::new(&params.input_dir)
                .join(format!("{}.rar", directory.file_name().to_string_lossy()))
        };

        let output_filename = output_filepath.file_name().unwrap().to_string_lossy();
        bar.set_message(format!("Compressing {}", &output_filename));
        let mut entries = WalkDir::new(directory.path())
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|v| !is_hidden(v) && !is_parent(v.path(), &directory.path().to_string_lossy()))
            .map(|v| v.file_name().to_string_lossy().to_string())
            .collect::<Vec<String>>();

        let mut args = ["a", "-r", "-m5", "--"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        args.push(output_filepath.to_string_lossy().to_string());
        args.append(&mut entries);

        let mut command = Command::new(RAR_PATH);
        command.args(args);
        command.current_dir(directory.path().to_string_lossy().to_string());

        match command.execute() {
            Ok(Some(exit_code)) => {
                if exit_code == 0 {
                    compress_success_files.insert(output_filename.to_string(), false);
                    bar.set_message(format!("Compressed {}!", &output_filename));
                } else {
                    compress_error_files.push(output_filename.to_string());
                    bar.set_message(format!("Failed to compress {}!", &output_filename));
                }
            }
            _ => {
                compress_error_files.push(output_filename.to_string());
                bar.set_message(format!("Failed to compress {}!", &output_filename));
            }
        };

        bar.inc(1);
    }
    bar.finish();

    // Show compression result
    println!("{}", "Compression Result".green().bold());
    println!("# ----------------- #");
    println!(
        "| {} |",
        format!("Total    ->  {: >4}", directories.len())
            .blue()
            .bold()
    );
    println!(
        "| {} |",
        format!("Created  ->  {: >4}", compress_success_files.len())
            .green()
            .bold()
    );
    println!(
        "| {} |",
        format!("Error    ->  {: >4}", compress_error_files.len())
            .red()
            .bold()
    );
    println!("# ----------------- #");

    // Show compress error directories
    if !compress_error_files.is_empty() {
        println!("{}", "The error directories are listed below".red().bold());
        for error_file in compress_error_files {
            println!("{}", error_file);
        }
    }

    // Validate compressed files
    if params.validate {
        let output_dir = if let Some(v) = &params.output_dir {
            v
        } else {
            &params.input_dir
        };
        validate_files(output_dir, compress_success_files);
    }
}

/// Validate files
///
/// # Arguments
///
/// * `current_dir` - Current directory
/// * `files` - Filepaths to validate
fn validate_files(current_dir: &str, mut files: HashMap<String, bool>) {
    let bar = get_progress_bar(files.len() as u64);

    for (filename, compress_success) in files.iter_mut() {
        let mut command = Command::new(RAR_PATH);
        command.args(vec!["t", "--", filename.as_str()]);
        command.current_dir(current_dir);

        bar.set_message(format!("Validating {}", filename));

        match command.execute() {
            Ok(Some(exit_code)) => {
                if exit_code == 0 {
                    *compress_success = true;
                    bar.set_message("OK");
                } else {
                    bar.set_message("NG");
                }
            }
            _ => {
                bar.set_message("NG");
            }
        }

        bar.inc(1);
    }
    bar.finish();

    let invalid_files: HashMap<_, _> = files.iter().filter(|&(_, valid)| !(*valid)).collect();
    let valid_files_len = files.len() - invalid_files.len();

    // Show validation result
    println!("{}", "Validation Result".green().bold());
    println!("# ----------------- #");
    println!(
        "| {} |",
        format!("Total    ->  {: >4}", files.len()).blue().bold()
    );
    println!(
        "| {} |",
        format!("Valid    ->  {: >4}", valid_files_len)
            .green()
            .bold()
    );
    println!(
        "| {} |",
        format!("Invalid  ->  {: >4}", invalid_files.len())
            .red()
            .bold()
    );
    println!("# ----------------- #");

    if !invalid_files.is_empty() {
        println!("{}", "The corrupted files are listed below".red().bold());
        for &invalid_file in invalid_files.keys() {
            println!("{}", invalid_file);
        }
    }
}
