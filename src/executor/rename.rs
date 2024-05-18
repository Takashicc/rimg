use crate::{
    executor::utils::{ask, gen_random_path, get_progress_bar, is_dir, is_file, is_hidden},
    params::rename::RenameParams,
};
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

        sort_natural(&mut files);

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
                files[index].clone_from(&random_path);
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

fn sort_natural(files: &mut [PathBuf]) {
    files.sort_by(|a, b| {
        let a_key = generate_natural_sort_key(a.to_string_lossy().to_string().as_str());
        let b_key = generate_natural_sort_key(b.to_string_lossy().to_string().as_str());

        let mut cmp_result = std::cmp::Ordering::Equal;
        for (a_part, b_part) in a_key.iter().zip(b_key.iter()) {
            let text_cmp_result = a_part.text.cmp(&b_part.text);
            if text_cmp_result != std::cmp::Ordering::Equal {
                cmp_result = text_cmp_result;
                break;
            }

            let number_cmp_result = a_part.number.cmp(&b_part.number);
            if number_cmp_result != std::cmp::Ordering::Equal {
                cmp_result = number_cmp_result;
                break;
            }
        }

        cmp_result
    })
}

struct NaturalSortKeyPart {
    text: String,
    number: Option<usize>,
}

fn generate_natural_sort_key(s: &str) -> Vec<NaturalSortKeyPart> {
    let mut key = vec![];
    let mut last = 0;
    for (start, end) in extract_number_positions(s) {
        key.push(NaturalSortKeyPart {
            text: s[last..start].to_string(),
            number: s[start..end].parse::<usize>().ok(),
        });
        last = end;
    }

    key.push(NaturalSortKeyPart {
        text: s[last..].to_string(),
        number: None,
    });

    key
}

fn extract_number_positions(s: &str) -> Vec<(usize, usize)> {
    let mut positions = vec![];
    let mut start_index = None;

    for (i, c) in s.char_indices() {
        if c.is_ascii_digit() {
            if start_index.is_none() {
                start_index = Some(i);
            }
        } else if let Some(start) = start_index {
            positions.push((start, i));
            start_index = None;
        }
    }

    if let Some(start) = start_index {
        positions.push((start, s.len()))
    }

    positions
}
