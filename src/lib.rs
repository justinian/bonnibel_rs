use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::path::{Path, PathBuf};

use failure::{err_msg, Fail, ResultExt};
use serde::Deserialize;
use tera::{Context, Tera};

mod module;
use module::{Module, ModuleKind};

type Result<T> = std::result::Result<T, failure::Error>;

fn tera_failure(e: tera::Error) -> failure::Error {
    let mut base = failure::Context::new(e.to_string());

    let mut next: &dyn StdError = &e;
    while let Some(e) = next.source() {
        base = base.context(e.to_string());
        next = e;
    }
    base.into()
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    templates: PathBuf,

    #[serde(skip)]
    pub root: PathBuf,

    #[serde(default)]
    vars: HashMap<String, String>,

    pub modules: HashMap<String, Module>,

    #[serde(skip)]
    targets: HashMap<String, HashSet<String>>,
}

impl Project {
    pub fn load(filename: &Path) -> Result<Project> {
        let config = std::fs::read_to_string(filename).context("reading config file")?;

        let mut proj: Project = serde_yaml::from_str(&config).context("parsing config file")?;

        proj.root = std::fs::canonicalize(filename)
            .context("finding project root")?
            .parent()
            .unwrap()
            .to_path_buf();

        proj.update_dependencies();
        //println!("{:?}", proj);
        Ok(proj)
    }

    fn update_dependencies(&mut self) {
        // Start each target off with its list of roots
        for (name, module) in self.modules.iter() {
            if let ModuleKind::Executable { target } = &module.kind {
                self.targets
                    .entry(target.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(name.to_string());
            }
        }

        // Walk the dependency graph and build a list of all required modules for each target
        for (_, target_modules) in self.targets.iter_mut() {
            let mut open_list: Vec<String> = target_modules.drain().collect();

            while let Some(dep) = open_list.pop() {
                let module = &self.modules[&dep];
                target_modules.insert(dep);

                for subdep in module.depends.iter() {
                    if !target_modules.contains(subdep) {
                        open_list.push(subdep.to_string());
                    }
                }
            }
        }
    }

    pub fn init(&self, build_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(build_dir).context("creating build output directory")?;

        let mut template_path = self.templates.clone();
        template_path.push("*");

        let tera = Tera::new(template_path.to_str().unwrap())
            .map_err(tera_failure)
            .context("parsing templates")?;

        let build_files: Vec<PathBuf> = Vec::new();

        for (name, m) in &self.modules {
            for t in m.template_names(name.as_str()) {
                template_path.pop();
                template_path.push(t.as_str());
                if !template_path.exists() {
                    continue;
                }

                let mut build_file = build_dir.to_path_buf();
                build_file.push(format!("{}.ninja", name));

                let mut ctx = Context::new();
                ctx.insert("module", &m);
                ctx.insert("name", &name);
                ctx.insert("buildfile", &build_file);
                ctx.insert("vars", &self.vars);
                ctx.insert("depmods", &m.depmods(self)?);

                println!("{}", tera.render(t.as_str(), ctx).map_err(tera_failure)?);
            }
        }

        Ok(())
    }

    pub fn module(&self, name: &str) -> Result<&Module> {
        self.modules.get(name).ok_or(err_msg("couldn't find module"))
    }
}