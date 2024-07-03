use clap::Parser;
use gfar::cmd::convert;
use gfar::cmd::index;
use gfar::cmd::pav;
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
    /// convert gfa between gfa1.0 in gfa1.1
    convert {
        /// input  gfa
        #[arg(short = 'g', long = "gfa", required = true)]
        input: String,
        /// output gfa
        #[arg(short = 'o', long = "output", required = true)]
        output: String,
        /// w2p <True 1 else False 0>
        #[arg(short = 'i', required = true, default_value = "1")]
        i: String,
    },
    /// output pav matrix of  node list
    pav {
        /// input  gfa
        #[arg(short = 'g', long = "gfa", required = true)]
        gfa: String,
        /// input  node list
        #[arg(short = 'n', long = "node", required = true)]
        node: String,
        /// output pav matrix
        #[arg(short = 'o', long = "output", required = true)]
        output: String,
    },
    /// build index for gfa
    index {
        /// input  gfa
        #[arg(short = 'g', long = "gfa", required = true)]
        gfa: String,
    },
}

#[warn(unused_imports)]
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
        Subcli::convert { input, output, i } => {
            let i = i.parse::<u32>().unwrap();
            if i == 0 {
                convert::convert_1_1(input, output)?
            } else {
                convert::convert_1_0(input, output)?
            }
        }
        Subcli::pav { gfa, node, output } => pav::run(gfa, node, output)?,
        Subcli::index { gfa } => {
            index::build(&gfa)?;
        }
    };
    println!("{}", format!("Done!, gfar {}", VERSION));
    resource::gather_resources();
    Ok(())
}
