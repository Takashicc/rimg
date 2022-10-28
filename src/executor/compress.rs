use crate::constant::RAR_PATH;
use crate::executor::utils::{ask, is_dir, is_hidden, is_parent};
use crate::params::CompressParams;
use execute::Execute;
use indicatif::{ProgressBar, ProgressStyle};
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

    // TODO If validate_only is true, get all the compressed file
    // TODO else get all the directories
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
        command.current_dir(params.input_dir.as_str());
        command.stdout(Stdio::null());

        if let Some(exit_code) = command.execute().unwrap() {
            if exit_code == 0 {
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

    // TODO After compressing, validate the compressed files when validate flag is true
}
