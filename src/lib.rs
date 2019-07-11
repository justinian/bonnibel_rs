use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, failure::Fail)]
pub enum BonnibelError {
    #[fail(display = "opening file {:?}: {:?}", file, orig)]
    OpenFile {
        file: PathBuf,
        orig: std::io::Error
    },

    #[fail(display = "parsing config: {:?}", orig)]
    ParseFile {
        orig: serde_yaml::Error
    },
}

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(alias = "deps")]
    #[serde(default)]
    depends: Vec<String>,

    #[serde(default)]
    defines: Vec<String>,

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
        #[serde(default)]
        includes: Vec<PathBuf>,
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

    #[serde(default)]
    vars: HashMap<String, String>,

    pub modules: HashMap<String, Module>,

    #[serde(skip)]
    targets: HashMap<String, HashSet<String>>,
}

impl Project {
    pub fn load(filename: &Path) -> Result<Project, BonnibelError> {
        let config = std::fs::read_to_string(filename)
            .map_err(|e| BonnibelError::OpenFile {file: filename.to_path_buf(), orig: e})?;

        let mut proj: Project = serde_yaml::from_str(&config)
            .map_err(|e| BonnibelError::ParseFile {orig: e})?;

        // Start each target off with its list of roots
        for (name, module) in proj.modules.iter() {
            if let ModuleKind::Executable { target } = &module.kind {
                proj.targets.entry(target.to_string())
                    .or_insert(HashSet::new())
                    .insert(name.to_string());
            }
        }

        // Walk the dependency graph and build a list of all required modules for each target
        for (name, target_modules) in proj.targets.iter_mut() {
            let mut open_list: Vec<String> = target_modules.drain().collect();
            loop {
                if let Some(dep) = open_list.pop() {
                    let module = &proj.modules[&dep];
                    target_modules.insert(dep);

                    for subdep in module.depends.iter() {
                        if !target_modules.contains(subdep) {
                            open_list.push(subdep.to_string());
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Ok(proj)
    }
}
