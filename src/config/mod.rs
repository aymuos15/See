use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    exclude: Vec<String>,
}

pub struct Config {
    pub exclude_set: GlobSet,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exclude_set: GlobSet::empty(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        Self::load_from_file().unwrap_or_default()
    }

    fn load_from_file() -> Option<Self> {
        let config_path = dirs::config_dir()?.join("viewer/config.toml");
        let content = fs::read_to_string(&config_path).ok()?;
        let config_file: ConfigFile = toml::from_str(&content).ok()?;

        let exclude_set = build_globset(&config_file.exclude)?;

        Some(Self { exclude_set })
    }

    pub fn is_excluded(&self, path: &Path) -> bool {
        self.exclude_set.is_match(path)
    }
}

fn build_globset(patterns: &[String]) -> Option<GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }

    builder.build().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.is_excluded(Path::new("any/path")));
    }

    #[test]
    fn test_is_excluded_with_simple_glob() {
        let patterns = vec!["*.log".to_string()];
        let exclude_set = build_globset(&patterns).expect("Failed to build globset");
        let config = Config { exclude_set };

        assert!(config.is_excluded(Path::new("debug.log")));
        assert!(config.is_excluded(Path::new("build.log")));
        assert!(!config.is_excluded(Path::new("file.txt")));
    }

    #[test]
    fn test_is_excluded_with_directory_glob() {
        let patterns = vec!["target/**".to_string()];
        let exclude_set = build_globset(&patterns).expect("Failed to build globset");
        let config = Config { exclude_set };

        assert!(config.is_excluded(Path::new("target/debug/app")));
        assert!(config.is_excluded(Path::new("target/release")));
        assert!(!config.is_excluded(Path::new("src/main.rs")));
    }

    #[test]
    fn test_is_excluded_with_multiple_patterns() {
        let patterns = vec![
            "*.log".to_string(),
            "*.tmp".to_string(),
            "node_modules".to_string(),
        ];
        let exclude_set = build_globset(&patterns).expect("Failed to build globset");
        let config = Config { exclude_set };

        assert!(config.is_excluded(Path::new("app.log")));
        assert!(config.is_excluded(Path::new("temp.tmp")));
        assert!(config.is_excluded(Path::new("node_modules")));
        assert!(!config.is_excluded(Path::new("main.rs")));
    }

    #[test]
    fn test_is_excluded_with_invalid_patterns() {
        // Invalid glob patterns are silently ignored
        let patterns = vec!["[invalid".to_string(), "*.valid".to_string()];
        let exclude_set = build_globset(&patterns).expect("Failed to build globset");
        let config = Config { exclude_set };

        // Valid pattern works
        assert!(config.is_excluded(Path::new("file.valid")));
        // Invalid pattern doesn't match anything
        assert!(!config.is_excluded(Path::new("[invalid")));
    }

    #[test]
    fn test_build_globset_empty() {
        let patterns: Vec<String> = vec![];
        let exclude_set = build_globset(&patterns).expect("Failed to build globset");
        assert!(!exclude_set.is_match(Path::new("anything")));
    }

    #[test]
    fn test_build_globset_all_invalid() {
        let patterns = vec!["[invalid".to_string(), "{bad".to_string()];
        let result = build_globset(&patterns);
        // build_globset returns Some even with all invalid patterns
        assert!(result.is_some());
    }
}
