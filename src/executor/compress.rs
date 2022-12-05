use crate::constant::{RAR_PATH, ZIP_EXTENSION};
use crate::executor::utils::{ask, get_progress_bar, have_extension, is_dir, is_hidden, is_parent};
use crate::params::compress::CompressParams;
use colored::Colorize;
use core::panic;
use execute::Execute;
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use walkdir::{DirEntry, WalkDir};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

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

        validate_files(&params, files);
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

    ask(params.yes);

    let success_files = compress_files(params, &directories);
    if params.validate {
        validate_files(params, success_files);
    }
}

/// Compress the directories directly under the given directory
/// and returns the name of the successfully compressed files
///
/// # Arguments
///
/// * `params` - Compress params
/// * `directories` - Directories to compress
fn compress_files(params: &CompressParams, directories: &Vec<DirEntry>) -> HashMap<String, bool> {
    let execute_target_len = directories.len() as u64;
    println!("{} directories will be executed", execute_target_len);

    let bar = get_progress_bar(execute_target_len);
    let mut success_files = HashMap::<String, bool>::new();
    let mut error_files = Vec::<String>::new();

    match params.format_type.as_str() {
        RAR_PATH => compress_rar(
            directories,
            params,
            &bar,
            &mut success_files,
            &mut error_files,
        ),
        ZIP_EXTENSION => compress_zip(
            directories,
            params,
            &bar,
            &mut success_files,
            &mut error_files,
        ),
        _ => panic!(),
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
        format!("Created  ->  {: >4}", success_files.len())
            .green()
            .bold()
    );
    println!(
        "| {} |",
        format!("Error    ->  {: >4}", error_files.len())
            .red()
            .bold()
    );
    println!("# ----------------- #");

    // Show compress error directories
    if !error_files.is_empty() {
        println!("{}", "The error directories are listed below".red().bold());
        for error_file in error_files {
            println!("{}", error_file);
        }
    }

    return success_files;
}

/// Compress files to rar
///
/// # Arguments
///
/// * `directories` - Directories to compress
/// * `params` - Compress params
/// * `bar` - Progress bar
/// * `success_files` - Successfully created files
/// * `error_files` - Error files
fn compress_rar(
    directories: &Vec<DirEntry>,
    params: &CompressParams,
    bar: &ProgressBar,
    success_files: &mut HashMap<String, bool>,
    error_files: &mut Vec<String>,
) {
    for directory in directories {
        let output_filepath = _get_output_filepath(params, directory, RAR_PATH);

        let output_filename = output_filepath.file_name().unwrap().to_string_lossy();
        bar.set_message(format!("Compressing {}", &output_filename));
        let mut entries = _get_string_entries(directory);

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
                    success_files.insert(output_filename.to_string(), false);
                    bar.set_message(format!("Compressed {}!", &output_filename));
                } else {
                    error_files.push(output_filename.to_string());
                    bar.set_message(format!("Failed to compress {}!", &output_filename));
                }
            }
            _ => {
                error_files.push(output_filename.to_string());
                bar.set_message(format!("Failed to compress {}!", &output_filename));
            }
        };

        bar.inc(1);
    }
}

/// Compress files to zip
///
/// # Arguments
///
/// * `directories` - Directories to compress
/// * `params` - Compress params
/// * `bar` - Progress bar
/// * `success_files` - Successfully created files
/// * `error_files` - Error files
fn compress_zip(
    directories: &Vec<DirEntry>,
    params: &CompressParams,
    bar: &ProgressBar,
    success_files: &mut HashMap<String, bool>,
    error_files: &mut Vec<String>,
) {
    for directory in directories {
        let output_filepath = _get_output_filepath(params, directory, ZIP_EXTENSION);
        let output_filename = output_filepath.file_name().unwrap().to_string_lossy();
        bar.set_message(format!("Compressing {}", &output_filename));

        let output_file = File::create(output_filepath).unwrap();
        let mut zip = ZipWriter::new(output_file);
        let zip_options = FileOptions::default()
            .compression_method(CompressionMethod::Bzip2)
            .unix_permissions(0o755);

        let entries = _get_path_entries(directory);
        // TODO Update success_files, error_files
        for entry in entries {
            let entry_filename = entry
                .strip_prefix(directory.path())
                .unwrap()
                .to_string_lossy();

            if entry.is_file() {
                // If entry is file
                zip.start_file(entry_filename, zip_options).unwrap();
                let mut f = File::open(entry).unwrap();
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&*buffer).unwrap();
                buffer.clear();
            } else if !entry_filename.is_empty() {
                // If entry is directory and the name is not empty
                zip.add_directory(entry_filename, zip_options).unwrap();
            }
        }

        zip.finish().unwrap();
    }
}

/// Returns output filepath
/// If the ouput_dir was specified, use output_dir for the output filepath
/// Otherwise, input_dir is used
///
/// # Arguments
///
/// * `params` - Compress params
/// * `directory` - Directory
/// * `extension` - Extension
fn _get_output_filepath(params: &CompressParams, directory: &DirEntry, extension: &str) -> PathBuf {
    let output_filepath = if let Some(v) = &params.output_dir {
        Path::new(&v).join(format!(
            "{}.{}",
            directory.file_name().to_string_lossy(),
            extension
        ))
    } else {
        Path::new(&params.input_dir).join(format!(
            "{}.{}",
            directory.file_name().to_string_lossy(),
            extension
        ))
    };

    output_filepath
}

/// Returns files/directories directly under the given directory
///
/// # Note
/// This will **NOT** look recursively
///
/// # Arguments
///
/// * `directory` - Directory
fn _get_string_entries(directory: &DirEntry) -> Vec<String> {
    WalkDir::new(directory.path())
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|v| !is_hidden(v) && !is_parent(v.path(), &directory.path().to_string_lossy()))
        .map(|v| v.file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>()
}

/// Returns files/directories directly under the given directory
///
/// # Note
/// This will look recursively
///
/// # Arguments
///
/// * `directory` - Directory
fn _get_path_entries(directory: &DirEntry) -> Vec<PathBuf> {
    WalkDir::new(directory.path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|v| !is_hidden(v) && !is_parent(v.path(), &directory.path().to_string_lossy()))
        .map(|v| v.path().to_owned())
        .collect::<Vec<PathBuf>>()
}

/// Validate files
///
/// # Arguments
///
/// * `params` - Compress params
/// * `files` - Filepaths to validate
fn validate_files(params: &CompressParams, mut files: HashMap<String, bool>) {
    let output_dir = if let Some(v) = &params.output_dir {
        v
    } else {
        &params.input_dir
    };

    let bar = get_progress_bar(files.len() as u64);

    match params.format_type.as_str() {
        RAR_PATH => {
            validate_rar(&mut files, &output_dir, &bar);
        }
        _ => panic!(),
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

/// Validate rar files
///
/// # Arguments
///
/// * `files` - Filepaths to validate
/// * `current_dir` - Current directory
/// * `bar` - Progress bar
fn validate_rar(files: &mut HashMap<String, bool>, current_dir: &str, bar: &ProgressBar) {
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
}
