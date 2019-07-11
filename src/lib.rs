use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, failure::Fail)]
pub enum BonnibelError {
    #[fail(display = "Error opening file")]
    OpenFile(std::io::Error),

    #[fail(display = "Error parsing config")]
    ParseFile(serde_yaml::Error),
}

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(alias = "deps")]
    depends: Option<Vec<String>>,
    defines: Option<HashMap<String, String>>,
    output: PathBuf,
    source: Vec<String>,

    #[serde(flatten)]
    kind: ModuleKind,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
pub enum ModuleKind {
    #[serde(rename = "lib")]
    Library {
        includes: Option<Vec<PathBuf>>,
    },

    #[serde(rename = "exe")]
    Executable {
        target: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    templates: PathBuf,
    vars: Option<HashMap<String, String>>,
    pub modules: HashMap<String, Module>,
}

impl Project {
    pub fn load(filename: &Path) -> Result<Project, BonnibelError> {
        let config = std::fs::read_to_string(filename)
            .map_err(BonnibelError::OpenFile)?;

        let proj: Project = serde_yaml::from_str(&config)
            .map_err(BonnibelError::ParseFile)?;

        Ok(proj)
    }
}
