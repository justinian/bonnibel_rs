use std::path::PathBuf;
use log::{trace, info};

use exitfailure::ExitFailure;
use structopt::StructOpt;

use bonnibel::Project;

#[derive(Debug, StructOpt)]
#[structopt(name = "Bonnibel")]
struct Bonnibel {
    /// The modules file to read from (default "modules.yaml")
    #[structopt(parse(from_os_str), short = "f", long = "file")]
    config_file: Option<PathBuf>,

    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Initialize the build directory
    #[structopt(name = "init")]
    InitCommand {
        /// Initialize a build in release mode
        #[structopt(short = "r", long = "release")]
        release: bool,
    },

    /// Synchronize external packages
    #[structopt(name = "sync")]
    SyncCommand {
    },

    /// Run the build via Ninja
    ///
    /// This command is mainly a shortcut for invoking Ninja to run the build.
    #[structopt(name = "build")]
    BuildCommand {
        /// Build in release mode
        #[structopt(short = "r", long = "release")]
        release: bool,
    },
}

fn main() -> Result<(), ExitFailure> {
    let opts = Bonnibel::from_args();
    opts.verbose.setup_env_logger("pb")?;
    trace!("Set up logging for bonnibel");

    let path = match opts.config_file {
        Some(name) => name,
        None => PathBuf::from("modules.yaml"),
    };

    let proj = Project::load(&path)?;
    info!("Loaded project `{}` with {} modules.", proj.name, proj.modules.len());

    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(indicatif::ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:50.cyan/blue} {pos:>4}/{len:4} {msg}")
        .progress_chars("##-"));

    /*
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
fn test_a_thing() {
}
