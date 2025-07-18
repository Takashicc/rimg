use crate::constants::file::{RAR_EXTENSION, ZIP_EXTENSION};
use crate::executor::utils::{ask, get_progress_bar, have_extension, is_dir, is_hidden, is_parent};
use crate::params::compress::CompressParams;
use colored::Colorize;
use execute::Execute;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::sync::{Arc, Mutex};
use walkdir::{DirEntry, WalkDir};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Compress each directory and validate
///
/// # Arguments
///
/// * `params` - Compress params
pub fn execute(params: &CompressParams) {
    // Check rar executable
    if params.format_type == RAR_EXTENSION
        && Command::new(RAR_EXTENSION)
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

        validate_files(params, &files);
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
        eprintln!("Abort...");
        process::exit(0);
    }

    ask(params.yes);

    let success_files = compress_files(params, &directories);
    if params.validate {
        validate_files(params, &success_files);
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
    println!("{execute_target_len} directories will be executed");

    let bar = get_progress_bar(execute_target_len);

    let success_files = Arc::new(Mutex::new(HashMap::<String, bool>::new()));
    let error_files = Arc::new(Mutex::new(Vec::<String>::new()));

    match params.format_type.as_str() {
        RAR_EXTENSION => compress_rar(directories, params, &bar, &success_files, &error_files),
        ZIP_EXTENSION => compress_zip(directories, params, &bar, &success_files, &error_files),
        _ => unimplemented!(),
    }

    bar.finish();

    // Show compression result
    let success_files_clone = success_files.lock().unwrap().clone();
    let error_files_clone = error_files.lock().unwrap().clone();

    println!(
        "Compression result: {}/{}/{} = {}/{}/{}",
        "Total".blue().bold(),
        "Success".green().bold(),
        "Error".red().bold(),
        format!("{execute_target_len}").blue().bold(),
        format!("{}", success_files_clone.len()).green().bold(),
        format!("{}", error_files_clone.len()).red().bold(),
    );

    // Show compress error directories
    if !error_files_clone.is_empty() {
        println!("{}", "The error directories are listed below".red().bold());
        for error_file in error_files_clone {
            println!("{error_file}");
        }
    }

    success_files_clone
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
    success_files: &Arc<Mutex<HashMap<String, bool>>>,
    error_files: &Arc<Mutex<Vec<String>>>,
) {
    directories.par_iter().for_each(|directory| {
        let output_filepath = _get_output_filepath(params, directory, RAR_EXTENSION);
        let output_filename = output_filepath.file_name().unwrap().to_string_lossy();
        bar.set_message(format!("Compressing {}", &output_filename));

        let mut args = ["a", "-r", "-m5", "--"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        args.push(output_filepath.to_string_lossy().to_string());
        let mut entries = _get_string_entries(directory);
        args.append(&mut entries);

        let mut command = Command::new(RAR_EXTENSION);
        command.args(args);
        command.current_dir(directory.path().to_string_lossy().to_string());

        match command.execute() {
            Ok(Some(exit_code)) => {
                if exit_code == 0 {
                    let mut success_files = success_files.lock().unwrap();
                    success_files.insert(output_filename.to_string(), false);
                    bar.set_message(format!("Compressed {}!", &output_filename));
                } else {
                    let mut error_files = error_files.lock().unwrap();
                    error_files.push(output_filename.to_string());
                    bar.set_message(format!("Failed to compress {}!", &output_filename));
                }
            }
            _ => {
                let mut error_files = error_files.lock().unwrap();
                error_files.push(output_filename.to_string());
                bar.set_message(format!("Failed to compress {}!", &output_filename));
            }
        };

        bar.inc(1);
    });
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
    success_files: &Arc<Mutex<HashMap<String, bool>>>,
    error_files: &Arc<Mutex<Vec<String>>>,
) {
    for directory in directories {
        let output_filepath = _get_output_filepath(params, directory, ZIP_EXTENSION);
        let output_filename = output_filepath.file_name().unwrap().to_string_lossy();
        bar.set_message(format!("Compressing {}", &output_filename));

        let output_file = match File::create(&output_filepath) {
            Ok(v) => v,
            Err(_) => {
                let mut error_files = error_files.lock().unwrap();
                error_files.push(output_filename.to_string());
                continue;
            }
        };
        let mut zip = ZipWriter::new(output_file);
        let zip_options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Bzip2)
            .unix_permissions(0o755);

        let entries = _get_path_entries(directory);
        for entry in entries {
            let entry_filename = match entry.strip_prefix(directory.path()) {
                Ok(v) => v.to_string_lossy(),
                Err(_) => {
                    let mut error_files = error_files.lock().unwrap();
                    error_files.push(output_filename.to_string());
                    continue;
                }
            };

            if entry.is_file() {
                // If entry is file
                if zip.start_file(entry_filename, zip_options).is_err() {
                    let mut error_files = error_files.lock().unwrap();
                    error_files.push(output_filename.to_string());
                    continue;
                }
                let mut f = match File::open(entry) {
                    Ok(v) => v,
                    Err(_) => {
                        let mut error_files = error_files.lock().unwrap();
                        error_files.push(output_filename.to_string());
                        continue;
                    }
                };
                let mut buffer = Vec::new();
                if f.read_to_end(&mut buffer).is_err() {
                    let mut error_files = error_files.lock().unwrap();
                    error_files.push(output_filename.to_string());
                    continue;
                }
                if zip.write_all(&buffer).is_err() {
                    let mut error_files = error_files.lock().unwrap();
                    error_files.push(output_filename.to_string());
                    continue;
                }
                buffer.clear();
            } else if entry.is_dir() {
                // If entry is directory
                if zip.add_directory(entry_filename, zip_options).is_err() {
                    let mut error_files = error_files.lock().unwrap();
                    error_files.push(output_filename.to_string());
                    continue;
                }
            }
        }

        if zip.finish().is_ok() {
            let mut success_files = success_files.lock().unwrap();
            success_files.insert(output_filename.to_string(), false);
        } else {
            let mut error_files = error_files.lock().unwrap();
            error_files.push(output_filename.to_string());
        }
    }
}

/// Returns output filepath
/// If the output_dir was specified, use output_dir for the output filepath
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
fn validate_files(params: &CompressParams, files: &HashMap<String, bool>) {
    let output_dir = if let Some(v) = &params.output_dir {
        v
    } else {
        &params.input_dir
    };

    let bar = get_progress_bar(files.len() as u64);

    let validation_result = match params.format_type.as_str() {
        RAR_EXTENSION => validate_rar(files, output_dir, &bar),
        ZIP_EXTENSION => validate_zip(files, output_dir, &bar),
        _ => unimplemented!(),
    };

    bar.finish();

    let invalid_files: HashMap<_, _> = validation_result
        .iter()
        .filter(|&(_, valid)| !(*valid))
        .collect();
    let valid_files_len = validation_result.len() - invalid_files.len();
    let invalid_files_len = invalid_files.len();

    // Show validation result
    println!(
        "Validation result: {}/{}/{} = {}/{}/{}",
        "Total".blue().bold(),
        "Valid".green().bold(),
        "Invalid".red().bold(),
        format!("{}", validation_result.len()).blue().bold(),
        format!("{valid_files_len}").green().bold(),
        format!("{invalid_files_len}").red().bold()
    );

    if !invalid_files.is_empty() {
        println!("{}", "The corrupted files are listed below".red().bold());
        for &invalid_file in invalid_files.keys() {
            println!("{invalid_file}");
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
///
/// # Returns
///
/// A new HashMap containing the validation result
fn validate_rar(
    files: &HashMap<String, bool>,
    current_dir: &str,
    bar: &ProgressBar,
) -> HashMap<String, bool> {
    let validated_files = Arc::new(Mutex::new(files.clone()));

    files.par_iter().for_each(|(filename, _)| {
        let mut command = Command::new(RAR_EXTENSION);
        command.args(vec!["t", "--", filename.as_str()]);
        command.current_dir(current_dir);

        bar.set_message(format!("Validating {filename}"));

        let is_valid = match command.execute() {
            Ok(Some(exit_code)) => {
                if exit_code == 0 {
                    bar.set_message("OK");
                    true
                } else {
                    bar.set_message("NG");
                    false
                }
            }
            _ => {
                bar.set_message("NG");
                false
            }
        };

        let mut validated_files = validated_files.lock().unwrap();
        validated_files.insert(filename.to_string(), is_valid);
        bar.inc(1);
    });

    let validated_files = validated_files.lock().unwrap().clone();
    validated_files
}

/// Validate zip files
///
/// # Arguments
///
/// * `files` - Filepaths to validate
/// * `current_dir` - Current directory
/// * `bar` - Progress bar
///
/// # Returns
///
/// A new HashMap containing the validation result
fn validate_zip(
    files: &HashMap<String, bool>,
    current_dir: &str,
    bar: &ProgressBar,
) -> HashMap<String, bool> {
    let validated_files = Arc::new(Mutex::new(files.clone()));

    files.par_iter().for_each(|(filename, _)| {
        let fullpath = Path::new(current_dir).join(filename);
        bar.set_message(format!("Validating {filename}"));

        let is_valid = match validate_zip_entries(&fullpath) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("filename: {filename}, error: {e}");
                false
            }
        };

        let mut validated_files = validated_files.lock().unwrap();
        validated_files.insert(filename.to_string(), is_valid);
        bar.inc(1);
    });

    let validated_files = validated_files.lock().unwrap().clone();
    validated_files
}

fn validate_zip_entries(filepath: &PathBuf) -> Result<bool, String> {
    let file = match File::open(filepath) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    let mut buffer = vec![0; 1024];
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };

        loop {
            match file.read_exact(&mut buffer) {
                Ok(_) => continue,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    Ok(true)
}
