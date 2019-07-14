use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Project;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "PathBuf")]
pub struct SourceItem {
    input: PathBuf,
}

impl From<PathBuf> for SourceItem {
    fn from(s: PathBuf) -> Self {
        SourceItem { input: s }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Module {
    #[serde(alias = "deps")]
    #[serde(default)]
    pub depends: Vec<String>,

    #[serde(default)]
    defines: Vec<String>,

    source: Vec<SourceItem>,

    output: PathBuf,

    #[serde(flatten)]
    pub kind: ModuleKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ModuleKind {
    #[serde(rename = "lib")]
    Library {
        #[serde(default)]
        includes: Vec<PathBuf>,
    },

    #[serde(rename = "exe")]
    Executable { target: String },
}

impl Module {
    pub fn template_names(&self, name: &str) -> Vec<String> {
        vec![
            format!("{}.{}.j2", self.kind.file_name(), name),
            format!("{}.default.j2", self.kind.file_name()),
        ]
    }

    pub fn depmods<'a>(&self, proj: &'a Project) -> Result<Vec<&'a Module>> {
        let mut v = Vec::new();
        for name in self.depends.iter() {
            v.push(proj.module(&name.as_str())?);
        }
        Ok(v)
    }
}

impl ModuleKind {
    fn file_name(&self) -> &str {
        match self {
            ModuleKind::Executable { .. } => "exe",
            ModuleKind::Library { .. } => "lib",
        }
    }
}
