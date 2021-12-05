use {
    serde_json::Value,
    std::{
        fs,
        io::Write,
        path::PathBuf,
    },
};

fn available_path(name: &str) -> PathBuf {
    let mut count = 1;
    let ext = "json";
    loop {
        let cmp = if count > 1 {
            format!("-{}", count)
        } else {
            "".to_string()
        };
        let file_name = format!(
            "{}-{}{}.{}",
            chrono::Local::now().format("%F-%Hh%M"),
            name,
            cmp,
            ext,
        );
        let path = PathBuf::from(file_name);
        if !path.exists() {
            return path;
        }
        count += 1;
    }
}

/// write a JSON value in a file whose name will be based on the provided
/// name, with a date and if necessary with an additional number to avoid
/// collision.
pub fn write_in_file(
    name: &str,
    value: &Value,
) -> anyhow::Result<PathBuf> {
    let path = available_path(name);
    let mut file = fs::File::create(&path)?;
    let json = serde_json::to_string_pretty(value)?;
    write!(&mut file, "{}\n", json)?;
    Ok(path)
}
