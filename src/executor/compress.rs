use crate::constant::RAR_PATH;
use crate::executor::utils::{ask, have_extension, is_dir, is_hidden, is_parent};
use crate::params::compress::CompressParams;
use colored::Colorize;
use execute::Execute;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::process::{self, Command, Stdio};
use walkdir::{DirEntry, WalkDir};

pub fn execute(params: &CompressParams) {
    // Check rar executable
    if params.format_type == RAR_PATH
        && Command::new(RAR_PATH)
            .execute_check_exit_status_code(0)
            .is_err()
    {
        println!("rar executable not found!\nAbort...");
        process::exit(1);
    }

    if params.validate_only {
        let files = WalkDir::new(&params.input_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|v| have_extension(&params.format_type, v.path()))
            .map(|v| (v.file_name().to_str().unwrap().to_owned(), false))
            .collect::<HashMap<String, bool>>();

        if files.is_empty() {
            println!(
                "There are no {} files to be executed\nAbort...",
                params.format_type.to_uppercase()
            );
            process::exit(0);
        }

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
        println!("There are no directories to be executed\nAbort...");
        process::exit(0);
    }

    let execute_target_len = directories.len();
    println!("{} directories will be executed", execute_target_len);

    ask(params.yes);

    let bar = ProgressBar::new(execute_target_len as u64).with_style(
        ProgressStyle::default_bar()
            .template("|{bar:60.green/blue}| {pos:5}/{len:5} {msg}")
            .unwrap()
            .progress_chars("##>-"),
    );

    let mut compressed_files = HashMap::<String, bool>::new();
    for directory in directories {
        let filename = format!("{}.rar", directory.path().to_str().unwrap());
        bar.set_message(format!("Compressing {}", &filename));
        let mut entries = WalkDir::new(directory.path().to_str().unwrap())
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|v| !is_hidden(v) && !is_parent(v.path(), directory.path().to_str().unwrap()))
            .map(|v| v.file_name().to_str().unwrap().to_owned())
            .collect::<Vec<String>>();

        let mut args = ["a", "-r", "-m5", "--"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        args.push(filename.clone());
        args.append(&mut entries);

        let mut command = Command::new(RAR_PATH);
        command.args(args);
        command.current_dir(directory.path().to_str().unwrap());
        command.stdout(Stdio::null());

        if let Some(exit_code) = command.execute().unwrap() {
            if exit_code == 0 {
                compressed_files.insert(filename.clone(), false);
                bar.set_message(format!("Compressed {}!", &filename));
            } else {
                bar.set_message(format!("Failed to compress {}!", &filename));
            }
        } else {
            bar.set_message("Interrupted!");
        }

        bar.inc(1);
    }
    bar.finish();

    // Validate compressed files
    if params.validate {
        validate_files(&params.input_dir, compressed_files);
    }
}

fn validate_files(input_dir: &str, mut files: HashMap<String, bool>) {
    let bar = ProgressBar::new(files.len() as u64).with_style(
        ProgressStyle::default_bar()
            .template("|{bar:60.green/blue}| {pos:5}/{len:5} {msg}")
            .unwrap()
            .progress_chars("##>-"),
    );
    for (filename, compress_success) in files.iter_mut() {
        let mut command = Command::new(RAR_PATH);
        command.args(vec!["t", "--", filename.as_str()]);
        command.current_dir(input_dir);
        command.stdout(Stdio::null());

        bar.set_message(format!("Validating {}", filename));

        if let Some(exit_code) = command.execute().unwrap() {
            if exit_code == 0 {
                *compress_success = true;
                bar.set_message("OK");
            } else {
                bar.set_message("NG");
            }
        } else {
            bar.set_message("Interrupted!");
        }

        bar.inc(1);
    }
    bar.finish();

    let invalid_files: HashMap<_, _> = files.iter().filter(|&(_, valid)| !(*valid)).collect();
    if !invalid_files.is_empty() {
        println!("{}", "The corrupted files are listed below".red().bold());
        for &invalid_file in invalid_files.keys() {
            println!("{}", invalid_file);
        }
    } else {
        println!("{}", "All files can be unpacked".green().bold());
    }
}
