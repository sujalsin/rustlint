use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rules: Rules,
    #[serde(default)]
    pub paths: Paths,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rules {
    #[serde(default = "default_max_line_length")]
    pub max_line_length: usize,
    #[serde(default)]
    pub ignore_unused_variables: bool,
    #[serde(default = "default_true")]
    pub strict_pep8: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            max_line_length: default_max_line_length(),
            ignore_unused_variables: false,
            strict_pep8: default_true(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Paths {
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn default() -> Self {
        Self {
            rules: Rules::default(),
            paths: Paths::default(),
        }
    }
}

fn default_max_line_length() -> usize {
    88 // Using black's default
}

fn default_true() -> bool {
    true
}
