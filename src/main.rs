use clap::Parser;
use gfar::cmd::prune;
use gfar::error::CmdError;
use gfar::logging;
use gfar::resource;
use log;

const VERSION: &str = "version 0.1";
#[derive(Parser, Debug)]
#[command(
    author = "Zhang Yixing",
    version = VERSION,
    about = "gfar is a tool for converting GFA files",
    long_about = None
)]
struct Args {
    #[clap(subcommand)]
    command: Subcli,
}

#[derive(Parser, Debug)]
#[allow(non_camel_case_types)]
enum Subcli {
    /// remove complex regions from gfa
    prune {
        /// input Paths gfa
        #[arg(short = 'g', long = "gfa", required = true)]
        input: String,
        /// output Walks gfa
        #[arg(short = 'o', long = "output", required = true)]
        prefix: String,
    },
}

fn main() -> Result<(), CmdError> {

    logging::init_logging();
    // log::debug!("Main|This is a debug message");
    // log::info!("This is an info message");
    // log::warn!("This is a warning message");
    // log::error!("This is an error message");
    // log messages:
    // 2024/05/13 12:18 [DEBUG]main.rs:35   Main|This is a debug message
    // 2024/05/13 12:18 [INFO]main.rs:36   This is an info message
    // 2024/05/13 12:18 [WARN]main.rs:37   This is a warning message
    // 2024/05/13 12:18 [ERROR]main.rs:38   This is an error message
    let arg: Args = Args::parse();
    match arg.command {
        Subcli::prune { input, prefix } => {
            prune::prune_gfa(input, prefix)?;
        }
    }
    println!("{}", format!("Done!, gfar {}",VERSION));
    resource::gather_resources();
    Ok(())
}
