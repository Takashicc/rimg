use std::fs;

pub fn dir_exists(s: &str) -> Result<String, String> {
    let metadata = fs::metadata(s).map_err(|_| format!("`{}` isn't a directory", s))?;
    if !metadata.is_dir() {
        Err(format!("`{}` isn't a directory", s))
    } else {
        Ok(s.to_owned())
    }
}

pub fn is_positive_number(s: &str) -> Result<u8, String> {
    let digit = s
        .parse()
        .map_err(|_| format!("`{} isn't a number or positive number", s))?;
    if digit == 0 {
        Err(format!("`{}` isn't a positive number", &digit))
    } else {
        Ok(digit)
    }
}

pub fn start_from_zero(s: &str) -> Result<u8, String> {
    let digit = s
        .parse()
        .map_err(|_| format!("`{}` isn't a number or starts from zero", s))?;
    Ok(digit)
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

    mod is_positive_number {
        use super::super::*;

        #[test]
        #[should_panic]
        fn minus_one_should_panic() {
            is_positive_number("-1").unwrap();
        }

        #[test]
        #[should_panic]
        fn zero_should_panic() {
            is_positive_number("0").unwrap();
        }

        #[test]
        fn one_should_return_number() {
            let result = is_positive_number("1").unwrap();
            assert_eq!(1, result);
        }

        #[test]
        #[should_panic]
        fn string_should_panic() {
            is_positive_number("hello").unwrap();
        }
    }

    mod start_from_zero {
        use super::super::*;

        #[test]
        #[should_panic]
        fn minus_one_should_panic() {
            start_from_zero("-1").unwrap();
        }

        #[test]
        fn zero_should_return_number() {
            let result = start_from_zero("0").unwrap();
            assert_eq!(0, result);
        }

        #[test]
        fn one_should_return_number() {
            let result = start_from_zero("1").unwrap();
            assert_eq!(1, result);
        }
    }
}