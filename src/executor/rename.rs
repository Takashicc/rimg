use crate::{
    executor::utils::{ask, gen_random_path, get_progress_bar, is_dir, is_file, is_hidden},
    params::rename::RenameParams,
};
use human_sort::compare;
use std::process;
use std::{ffi::OsStr, fs, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Rename files in each directory
///
/// # Arguments
///
/// * `params` - Rename params
pub fn execute(params: &RenameParams) {
    let directories = WalkDir::new(&params.input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| is_dir(e) && !is_hidden(e))
        .collect::<Vec<DirEntry>>();

    let directories_count = directories.len();
    println!("{} directories will be executed", directories_count);

    ask(params.yes);

    for entry in directories {
        let mut files = WalkDir::new(entry.path())
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|v| {
                is_file(v)
                    && !is_hidden(v)
                    && params.extensions.contains(
                        &v.path()
                            .extension()
                            .unwrap_or_else(|| OsStr::new(""))
                            .to_string_lossy()
                            .to_lowercase(),
                    )
            })
            .map(|v| v.into_path())
            .collect::<Vec<PathBuf>>();

        // Sort with human-friendly order
        files.sort_by(|a, b| {
            let a = a.to_string_lossy().to_string();
            let b = b.to_string_lossy().to_string();
            compare(&a, &b)
        });

        let dir_name = entry.file_name().to_string_lossy();
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

        let bar = get_progress_bar(files.len() as u64);
        bar.set_message(format!("Renaming {}", dir_name));

        let mut seq_index = params.initial;
        for i in 0..files.len() {
            let from_path = &files[i].clone();
            let from_parent = from_path.parent().unwrap();
            let extension = from_path
                .extension()
                .unwrap_or_else(|| OsStr::new(""))
                .to_string_lossy()
                .to_string();

            let to_path = &from_parent.join(format!(
                "{:0width$}.{ext}",
                seq_index,
                width = params.digit as usize,
                ext = extension
            ));

            // If the file was already renamed, skip
            if from_path == to_path {
                seq_index += params.step as u32;
                bar.inc(1);
                continue;
            }

            // If the destination file already exists, rename the existed file to random filename
            if to_path.exists() {
                let random_path = gen_random_path(from_parent, &extension);
                let index = match files.iter().position(|v| v == to_path) {
                    Some(v) => v,
                    None => {
                        eprintln!("Unexpected error!");
                        process::exit(1);
                    }
                };
                files[index] = random_path.clone();
                file_rename(to_path, &random_path);
            }

            file_rename(from_path, to_path);
            seq_index += params.step as u32;
            bar.inc(1);
        }

        bar.set_message(format!("Rename complete {}", dir_name));
        bar.finish();
    }
}

/// Rename file until success
///
/// # Arguments
///
/// * `from_path` - From path
/// * `to_path` - To path
fn file_rename(from_path: &PathBuf, to_path: &PathBuf) {
    loop {
        if fs::rename(from_path, to_path).is_ok() {
            break;
        } else {
            eprintln!("Error! Renaming {:?} to {:?}", from_path, to_path);
            eprintln!("Try again...");
        }
    }
}
