use question::{Answer, Question};
use std::path::Path;
use std::path::PathBuf;
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
