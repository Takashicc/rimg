use crate::{
    executor::utils::{ask, gen_random_path, is_dir, is_file, is_hidden},
    params::RenameParams,
};
use human_sort::compare;
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, path::PathBuf};
use walkdir::{DirEntry, WalkDir};

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
