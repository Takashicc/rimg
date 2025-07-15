use crate::constants::file::{RAR_EXTENSION, ZIP_EXTENSION};
use std::fs;

/// Check dir exists or not
///
/// # Arguments
///
/// * `s` - Given arg
pub fn dir_exists(s: &str) -> Result<String, String> {
    let metadata = fs::metadata(s).map_err(|_| format!("`{s}` isn't a directory"))?;
    if !metadata.is_dir() {
        Err(format!("`{s}` isn't a directory"))
    } else {
        Ok(s.to_owned())
    }
}

/// Check format type is valid
///
/// # Arguments
///
/// * `s` - Given arg
pub fn format_type_check(s: &str) -> Result<String, String> {
    if s == RAR_EXTENSION || s == ZIP_EXTENSION {
        Ok(s.to_owned())
    } else {
        Err(format!(
            "`{s}` isn't supported format type\nCurrently supports `rar` and `zip`"
        ))
    }
}

/// Check extension is valid
///
/// # Arguments
///
/// * `s` - Given arg
pub fn extension_check(s: &str) -> Result<String, String> {
    if s.is_empty() {
        Err("Empty extension is not valid".to_string())
    } else if s.len() > 5 {
        Err(format!("`{s}` Extension is too long\nMax length is 4"))
    } else {
        Ok(s.to_owned())
    }
}

#[cfg(test)]
mod tests {

    mod dir_exists {
        use super::super::*;
        use std::path::PathBuf;

        #[test]
        fn existed_dir_should_return_path() {
            let path = env!("CARGO_MANIFEST_DIR");
            let result = dir_exists(path).unwrap();
            assert_eq!(path, result);
        }

        #[test]
        #[should_panic]
        fn non_existed_dir_should_panic() {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("dummy");
            dir_exists(String::from(path.to_string_lossy()).as_str()).unwrap();
        }
    }

    mod format_type_check {
        use super::super::*;

        #[test]
        fn valid_format_type_should_return_string() {
            let result = format_type_check("rar").unwrap();
            assert_eq!("rar", result);
        }

        #[test]
        #[should_panic]
        fn invalid_format_type_should_panic() {
            format_type_check("xxx").unwrap();
        }
    }

    mod extension_check {
        use super::super::*;

        #[test]
        #[should_panic]
        fn empty_extension_should_panic() {
            extension_check("").unwrap();
        }

        #[test]
        fn one_char_extension_should_return_string() {
            let result = extension_check("a").unwrap();
            assert_eq!("a", result);
        }

        #[test]
        fn five_char_extension_should_return_string() {
            let result = extension_check("abcde").unwrap();
            assert_eq!("abcde", result);
        }

        #[test]
        #[should_panic]
        fn six_char_extension_should_panic() {
            extension_check("abcdef").unwrap();
        }
    }
}
