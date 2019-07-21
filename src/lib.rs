use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error as StdError;
use std::io::Write;
use std::path::{Path, PathBuf};

use failure::{err_msg, Fail, format_err, ResultExt};
use git2::{DescribeFormatOptions, DescribeOptions, Repository};
use log::{debug, info, trace};
use semver::{Identifier, Version};
use serde::Deserialize;
use tera::{Context, Tera};

mod module;
use module::{Module, ModuleKind};

type Result<T> = std::result::Result<T, failure::Error>;

const VAR_FILE_NAME: &'static str = ".bonnibel_vars";

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

    #[serde(skip)]
    pub config_file: PathBuf,

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

        proj.config_file = std::fs::canonicalize(filename)
            .context("finding project path")?
            .to_path_buf();

        trace!("Parsed config file {:?}", proj.config_file);

        proj.root = proj.config_file
            .parent()
            .unwrap()
            .to_path_buf();

        debug!("Source root is {:?}", proj.root);

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

    pub fn parse_vars(&mut self, vars: Vec<String>) -> Result<()> {
        for mut name in vars {
            if let Some(index) = name.find('=') {
                let value = name.split_off(index+1);
                name.pop(); // Strip off the =
                self.vars.insert(name, value);
            } else {
                return Err(format_err!("Variable {} does not parse", name));
            }
        }

        Ok(())
    }

    pub fn load_vars(&mut self, build_dir: &Path) -> Result<()> {
        let mut var_file = build_dir.to_path_buf();
        var_file.push(VAR_FILE_NAME);

        let var_file = std::fs::read_to_string(var_file).context("reading state file")?;
        self.vars = serde_yaml::from_str(&var_file).context("parsing state file")?;
        debug!("Loaded existing state: {:?}", self.vars);

        Ok(())
    }

    pub fn initialize(&self, build_dir: &Path) -> Result<()> {
        info!("Creating build directory at: {:?}", build_dir);
        std::fs::create_dir_all(build_dir).context("creating build output directory")?;

        let mut var_file = build_dir.to_path_buf();
        var_file.push(VAR_FILE_NAME);

        let mut var_file = std::fs::File::create(var_file)?;
        serde_yaml::to_writer(&var_file, &self.vars)?;
        var_file.write(b"\n")?;

        Ok(())
    }

    pub fn generate(&self, build_dir: &Path) -> Result<()> {
        let version = get_version(&self.root)
            .context("Getting current version")?;

        println!("Generating build files for {} version {}", self.name, version);

        let mut template_path = self.root.clone();
        template_path.push(&self.templates);
        template_path.push("*");

        let tera = Tera::new(template_path.to_str().unwrap())
            .map_err(tera_failure)
            .context("parsing templates")?;

        template_path.pop();
        let mut build_files: Vec<PathBuf> = Vec::new();
        let mut templates: Vec<PathBuf> = Vec::new();

        for (name, m) in &self.modules {
            let (template_path, template_file) =
                template_from_options(&template_path, name, m.kind_name())?;

            let mut build_file = build_dir.to_path_buf();
            build_file.push(format!("{}.ninja", name));

            let mut build_file_out = std::fs::File::create(&build_file)
                .context("creating build file")?;

            let mut ctx = Context::new();
            ctx.insert("module", &m);
            ctx.insert("name", &name);
            ctx.insert("buildfile", &build_file);
            ctx.insert("vars", &self.vars);
            ctx.insert("depmods", &m.depmods(self)?);
            ctx.insert("deplibs", &m.deplibs(self)?);
            ctx.insert("depexes", &m.depexes(self)?);

            let contents = tera.render(template_file.as_str(), ctx)
                .map_err(tera_failure)?
                .into_bytes();

            build_file_out.write_all(&contents)
                .context("writing build file contents")?;

            build_files.push(build_file);
            templates.push(template_path);
        }

        for (target, mods) in &self.targets {
            let (template_path, template_file) =
                template_from_options(&template_path, target, "target")?;

            let mut target_root = build_dir.to_path_buf();
            target_root.push(target);

            std::fs::create_dir_all(&target_root)
                .context("creating target output directory")?;

            let mut build_file = target_root.to_path_buf();
            build_file.push("target.ninja");

            let mut ctx = Context::new();
            ctx.insert("target", &target);
            ctx.insert("modules", &mods);
            ctx.insert("buildfile", &build_file);
            ctx.insert("vars", &self.vars);

            let contents = tera.render(template_file.as_str(), ctx)
                .map_err(tera_failure)?
                .into_bytes();

            let mut build_file_out = std::fs::File::create(&build_file)
                .context("creating build file")?;

            build_file_out.write_all(&contents)
                .context("writing build file contents")?;

            build_files.push(build_file);
            templates.push(template_path);
        }

        template_path.push("build.ninja.j2");

        let mut build_file = build_dir.to_path_buf();
        build_file.push("build.ninja");

        build_files.push(build_file.clone());
        templates.push(template_path);

        let target_names: Vec<&String> = self.targets.keys().collect();

        let sha = format!("{}", version.build[0]);
        let version_string = format!("{}", version);

        let mut ctx = Context::new();
        ctx.insert("targets", &target_names);
        ctx.insert("modules", &self.modules);
        ctx.insert("vars", &self.vars);
        ctx.insert("buildroot", &build_dir);
        ctx.insert("srcroot", &self.root);
        ctx.insert("modulefile", &self.config_file);
        ctx.insert("version_major", &version.major );
        ctx.insert("version_minor", &version.minor );
        ctx.insert("version_patch", &version.patch );
        ctx.insert("version_sha", &sha);
        ctx.insert("version", &version_string);
        ctx.insert("buildfiles", &build_files);
        ctx.insert("templates", &templates);

        let generator = env::current_exe()
            .expect("Couldn't get current executable name!");
        ctx.insert("generator", &generator);
        debug!("Generator is {:?}", generator);

        // Top level buildfile cannot use an absolute path or ninja won't
        // reload itself properly on changes.
        // See: https://github.com/ninja-build/ninja/issues/1240
        ctx.insert("buildfile", "build.ninja");

        let contents = tera.render("build.ninja.j2", ctx)
            .map_err(tera_failure)?
            .into_bytes();

        let mut build_file_out = std::fs::File::create(&build_file)
            .context("creating build file")?;

        build_file_out.write_all(&contents)
            .context("writing build file contents")?;

        Ok(())
    }

    pub fn module(&self, name: &str) -> Result<&Module> {
        self.modules.get(name).ok_or(err_msg("couldn't find module"))
    }
}

fn template_from_options(root: &Path, name: &str, kind: &str) -> Result<(PathBuf, String)> {
    let mut path = root.to_path_buf();
    let file = format!("{}.{}.j2", kind, name);
    path.push(&file);

    if path.exists() {
        Ok((path, file))
    } else {
        let file = format!("{}.default.j2", kind);
        path.set_file_name(&file);
        if path.exists() {
            Ok((path, file))
        } else {
            Err(format_err!("Missing template for module '{}'.", name))
        }
    }
}

fn get_version(root: &Path) -> Result<Version> {
    let repo = Repository::open(root)
        .context("opening git repository")?;

    let desc = repo
        .describe(DescribeOptions::new().max_candidates_tags(100))
        .context("getting git description")?
        .format(Some(DescribeFormatOptions::new()
                     .abbreviated_size(0)
                     .dirty_suffix("+dirty")))
        .context("formatting git description")?;

    let mut v = Version::parse(desc.as_str())
        .unwrap_or_else(|_| Version::new(0,0,0));

    let head = repo
        .revparse_single("HEAD")
        .context("finding git head revision")?
        .short_id()
        .context("getting git head revision id")?;

    v.build.insert(0,
        Identifier::AlphaNumeric(head.as_str().unwrap().to_string()));
    Ok(v)
}
