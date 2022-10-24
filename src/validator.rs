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
