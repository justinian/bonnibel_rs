use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Project;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Module {
    #[serde(alias = "deps")]
    #[serde(default)]
    pub depends: Vec<String>,

    #[serde(default)]
    defines: Vec<String>,

    #[serde(default)]
    includes: Vec<PathBuf>,

    source: Vec<SourceItem>,

    output: PathBuf,

    #[serde(flatten)]
    pub kind: ModuleKind,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum ModuleKind {
    #[serde(rename = "lib")]
    Library,

    #[serde(rename = "exe")]
    Executable { target: String },
}

impl Module {
    pub fn depmods<'a>(&self, proj: &'a Project) -> Result<Vec<&'a Module>> {
        let mut open = self.depends.iter()
            .map(|n| proj.module(n))
            .collect::<Result<Vec<&'a Module>>>()?;

        let mut closed = HashSet::new();
        while let Some(module) = open.pop() {
            closed.insert(module);
            for dep in &module.depends {
                let dep = proj.module(&dep)?;
                if !closed.contains(dep) {
                    open.push(dep);
                }
            }
        }

        Ok(closed.into_iter().collect())
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

    pub fn kind_name(&self) -> &str {
        match self.kind {
            ModuleKind::Executable { .. } => "exe",
            ModuleKind::Library { .. } => "lib",
        }
    }
}
