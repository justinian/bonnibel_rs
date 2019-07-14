use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Project;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "PathBuf")]
pub struct SourceItem {
    name: String,
    input: PathBuf,
    output: PathBuf,
    action: String,
}

impl From<PathBuf> for SourceItem {
    fn from(s: PathBuf) -> Self {
        let action = s.extension().map(|s| s.to_str().unwrap()).unwrap_or("unknown").to_string();

        SourceItem {
            name: s.file_name().map(|s| s.to_str().unwrap()).unwrap_or("unnamed").to_string(),
            output: s.with_extension(format!("{}.o", action)),
            input: s,
            action: action,
        }
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

    pub fn deplibs<'a>(&self, proj: &'a Project) -> Result<Vec<&'a Module>> {
        Ok(self.depmods(proj)?
           .iter()
           .filter(|m| match m.kind { ModuleKind::Library { .. } => true, _ => false })
           .map(|m| *m)
           .collect())
    }

    pub fn depexes<'a>(&self, proj: &'a Project) -> Result<Vec<&'a Module>> {
        Ok(self.depmods(proj)?
           .iter()
           .filter(|m| match m.kind { ModuleKind::Executable { .. } => true, _ => false })
           .map(|m| *m)
           .collect())
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
