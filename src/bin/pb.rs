use std::path::PathBuf;
use std::process::Command as ExecCommand;

use directories::ProjectDirs;
use exitfailure::ExitFailure;
use failure::{err_msg, ResultExt};
use indicatif::{ProgressBar, ProgressStyle};
use structopt::StructOpt;

use bonnibel::Project;

#[derive(Debug, StructOpt)]
#[structopt(name = "Bonnibel")]
struct Bonnibel {
    /// The modules file to read from (default "modules.yaml")
    #[structopt(parse(from_os_str), short = "f", long = "file")]
    config_file: Option<PathBuf>,

    /// The build directory to use (default "build")
    #[structopt(parse(from_os_str), short = "d", long = "dir")]
    build_dir: Option<PathBuf>,

    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Initialize the build directory and options
    #[structopt(name = "init")]
    Init {
        /// A series of name=value pairs
        vars: Vec<String>,
    },

    /// Regenerate the build files
    #[structopt(name = "generate")]
    Regenerate,

    /// Synchronize external packages
    #[structopt(name = "sync")]
    Sync {
        /// Location of the download cache (default ~/.cache/bonnibel)
        #[structopt(parse(from_os_str), short = "c", long = "cache")]
        cache: Option<PathBuf>,
    },

    /// Run the build via Ninja
    ///
    /// This command is mainly a shortcut for invoking Ninja to run the build.
    #[structopt(name = "build")]
    Build,
}

fn main() -> Result<(), ExitFailure> {
    let proj_dirs = ProjectDirs::from("dev", "jsix", "bonnibel")
        .ok_or(err_msg("couldn't find home directory"))?;

    let opts = Bonnibel::from_args();
    opts.verbose.setup_env_logger("bonnibel")?;

    let path = match opts.config_file {
        Some(name) => name,
        None => PathBuf::from("modules.yaml"),
    };

    let mut proj = Project::load(&path)?;

    let build_dir = match opts.build_dir {
        Some(dir) => std::fs::canonicalize(dir)
            .context("finding build path")?
            .to_path_buf(),
        None => {
            let mut dir = proj.root.to_path_buf();
            dir.push("build");
            dir
        },
    };

    match opts.command {
        Command::Init { vars } => {
            proj.parse_vars(vars)?;
            proj.initialize(&build_dir)?;
            proj.generate(&build_dir)?;
        },

        Command::Regenerate => {
            proj.load_vars(&build_dir)?;
            proj.generate(&build_dir)?;
        },

        Command::Build => {
            ExecCommand::new("ninja")
                .arg("-C")
                .arg(&build_dir)
                .spawn()
                .context("Running ninja")?
                .wait()
                .context("Waiting for ninja child process")?;
        },

        Command::Sync { cache } => {
            let cache = match cache {
                Some(path) => path,
                None => proj_dirs.cache_dir().to_path_buf(),
            };

            for o in proj.overlays.iter_mut() {
                o.compute_for_cache(&cache)?;
                if !o.is_cached() {
                    let pb = ProgressBar::new(100);
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("[{elapsed:>3}] {bar:40.cyan/blue} {bytes:>8}/{total_bytes:8} {wide_msg}")
                            .progress_chars("##-"));

                    pb.println(format!("# Syncing {}", &o.filename));
                    pb.set_message("Downloading");

                    o.download(
                        |n| pb.set_length(n),
                        |n| pb.inc(n))?;
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    //pb.finish_and_clear();
                }
            }

        },
    }

    Ok(())
}

#[test]
fn test_a_thing() {}
