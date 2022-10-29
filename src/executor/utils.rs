use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use question::{Answer, Question};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process;
use uuid::Uuid;
use walkdir::DirEntry;

pub fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
}

pub fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn is_parent(v: &Path, parent: &str) -> bool {
    v == Path::new(parent)
}

pub fn have_extension(extension: &str, path: &Path) -> bool {
    if let Some(v) = path.extension() {
        v.to_str().unwrap() == extension
    } else {
        false
    }
}

pub fn gen_random_path(parent: &Path, ext: &str) -> PathBuf {
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

pub fn ask(yes: bool) {
    if !yes {
        let answer = Question::new("Are you sure to execute? (y/n):").confirm();
        if answer == Answer::NO {
            println!("Abort...");
            process::exit(0);
        }
    }
}

pub fn get_progress_bar(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos/len} ({eta}) {msg}",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    pb
}
