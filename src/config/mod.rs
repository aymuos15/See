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
