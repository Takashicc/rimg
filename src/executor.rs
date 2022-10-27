use crate::constant::RAR_PATH;
use crate::params::{CompressParams, RenameParams};
use execute::Execute;
use human_sort::compare;
use indicatif::{ProgressBar, ProgressStyle};
use question::{Answer, Question};
use std::path::Path;
use std::process::{self, Command, Stdio};
use std::{fs, path::PathBuf};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

pub fn rename(params: &RenameParams) {
    let directories = WalkDir::new(&params.input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_dir(e) && !is_hidden(e))
        .collect::<Vec<DirEntry>>();

    let directories_count = directories.len();
    println!("{} directories will be executed", directories_count);

    ask(params.yes);

    for entry in directories {
        let mut files = WalkDir::new(entry.path().to_str().unwrap())
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|v| {
                is_file(v)
                    && !is_hidden(v)
                    && params.extensions.contains(
                        &v.path()
                            .extension()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_lowercase(),
                    )
            })
            .map(|v| v.into_path())
            .collect::<Vec<PathBuf>>();

        // Sort with human-friendly order
        files.sort_by(|a, b| {
            let a = a.file_name().unwrap().to_str().unwrap();
            let b = b.file_name().unwrap().to_str().unwrap();
            compare(a, b)
        });

        let dir_name = entry.file_name().to_str().unwrap();
        let extension_types = params.extensions.join(", ").to_lowercase();
        if files.is_empty() {
            println!(
                "There are no {} files in {} directory",
                extension_types, dir_name
            );
            continue;
        } else {
            println!(
                "Renaming {} files in {} directory",
                extension_types, dir_name
            );
        }

        let mut template = String::from("|{bar:60.green/blue}| {pos:5}/{len:5} Renaming ");
        template.push_str(dir_name);
        let template = template.as_str();

        let bar = ProgressBar::new(files.len() as u64).with_style(
            ProgressStyle::default_bar()
                .template(template)
                .unwrap()
                .progress_chars("##>-"),
        );

        let mut seq_index = params.initial;
        for i in 0..files.len() {
            let from_path = &files[i].clone();
            let from_parent = from_path.parent().unwrap();
            let extension = from_path.extension().unwrap().to_str().unwrap();

            let to_path = &from_parent.join(format!(
                "{:0width$}.{ext}",
                seq_index,
                width = params.digit as usize,
                ext = extension
            ));

            // If the file was already renamed, skip
            if from_path == to_path {
                seq_index += params.step;
                bar.inc(1);
                continue;
            }

            // If the destination file already exists, rename the existed file to random filename
            if to_path.exists() {
                let random_path = gen_random_path(from_parent, extension);
                let index = files.iter().position(|v| v == to_path).unwrap();
                files[index] = random_path.clone();
                fs::rename(to_path, random_path).unwrap();
            }

            fs::rename(from_path, to_path).unwrap();
            seq_index += params.step;
            bar.inc(1);
        }

        bar.finish();
    }
}

pub fn compress(params: &CompressParams) {
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

fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_parent(v: &Path, parent: &str) -> bool {
    v == Path::new(parent)
}

fn gen_random_path(parent: &Path, ext: &str) -> PathBuf {
    let mut random_path;
    loop {
        let uuid = Uuid::new_v4().to_string();
        random_path = parent.join(format!("{}.{}", uuid, ext));
        if !random_path.exists() {
            break;
        }
    }

    random_path
}

fn ask(yes: bool) {
    if !yes {
        let answer = Question::new("Are you sure to execute? (y/n):").confirm();
        if answer == Answer::NO {
            println!("Abort...");
            process::exit(0);
        }
    }
}
