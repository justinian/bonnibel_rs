use std::path::PathBuf;

use exitfailure::ExitFailure;
use failure::ResultExt;
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
    Sync,

    /// Run the build via Ninja
    ///
    /// This command is mainly a shortcut for invoking Ninja to run the build.
    #[structopt(name = "build")]
    Build {
        /// Build in release mode
        #[structopt(short = "r", long = "release")]
        release: bool,
    },
}

fn main() -> Result<(), ExitFailure> {
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
        }

        Command::Regenerate => {
            proj.load_vars(&build_dir)?;
            proj.generate(&build_dir)?;
        }

        _ => {}
    }

    /*
    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>4}/{len:4} {msg}")
            .progress_chars("##-"),
    );

    for _ in 1..100 {
        std::thread::sleep(std::time::Duration::from_millis(10));
        //pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
    }
    pb.finish_with_message("done");
    */

    Ok(())
}

#[test]
fn test_a_thing() {}
