use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use question::{Answer, Question};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::process;
use uuid::Uuid;
use walkdir::DirEntry;

/// Returns true if the given entry is file
///
/// # Arguments
///
/// * `entry` - Entry
pub fn is_file(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
}

/// Returns true if the given entry is directory
///
/// # Arguments
///
/// * `entry` - Entry
pub fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
}

/// Returns true if the given entry is hidden file or directory
///
/// # Arguments
///
/// * `entry` - Entry
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy().starts_with('.')
}

/// Returns true if the given path and parent path is same
///
/// # Arguments
///
/// * `v` - Path
/// * `parent` - Parent path string
pub fn is_parent(v: &Path, parent: &str) -> bool {
    v == Path::new(parent)
}

/// Returns true if the given entry has the given extension
///
/// # Arguments
///
/// * `extension` - Extension
/// * `path` - Path
pub fn have_extension(extension: &str, path: &Path) -> bool {
    if let Some(v) = path.extension() {
        if let Some(v) = v.to_str() {
            return v == extension;
        }
    }

    false
}

/// Returns random path that does not exist in given parent path
///
/// # Arguments
///
/// * `parent` - Parent path
/// * `ext` - Extension
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

/// Ask the user to execute or not
///
/// # Arguments
///
/// * `yes` - Execute if yes is false
pub fn ask(yes: bool) {
    if !yes {
        let answer = Question::new("Are you sure to execute? (y/n):").confirm();
        if answer == Answer::NO {
            eprintln!("Abort...");
            process::exit(0);
        }
    }
}

/// Get progress bar
///
/// # Arguments
///
/// * `length` - Length for progress bar
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
