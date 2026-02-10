use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// `"path:weight"` 形式の文字列をパースする。
/// weight が省略された場合は 1.0 を返す。
///
/// 例: `"work/jawiki/:0.3"` → `("work/jawiki/", 0.3)`
///     `"work/jawiki/"` → `("work/jawiki/", 1.0)`
pub fn parse_dir_weight(s: &str) -> (String, f64) {
    if let Some(pos) = s.rfind(':') {
        let (path, weight_str) = s.split_at(pos);
        if let Ok(w) = weight_str[1..].parse::<f64>() {
            return (path.to_string(), w);
        }
    }
    (s.to_string(), 1.0)
}

pub fn get_file_list(src_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut result: Vec<PathBuf> = Vec::new();

    for src_file in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.metadata().unwrap().is_file())
    {
        result.push(src_file.path().to_path_buf());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dir_weight_with_weight() {
        let (path, weight) = parse_dir_weight("work/jawiki/:0.3");
        assert_eq!(path, "work/jawiki/");
        assert!((weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_dir_weight_without_weight() {
        let (path, weight) = parse_dir_weight("work/jawiki/");
        assert_eq!(path, "work/jawiki/");
        assert!((weight - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_dir_weight_with_colon_in_path() {
        // Windows-style path like "C:/Users/foo" should not be parsed as weight
        let (path, weight) = parse_dir_weight("C:/Users/foo");
        assert_eq!(path, "C:/Users/foo");
        assert!((weight - 1.0).abs() < f64::EPSILON);
    }
}

pub fn copy_snapshot(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all("work/dump/")?;
    fs::copy(
        path,
        Path::new("work/dump/").join(
            Local::now().format("%Y%m%d-%H%M%S").to_string()
                + path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .as_str(),
        ),
    )?;
    Ok(())
}
