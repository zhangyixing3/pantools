use clap::{Parser, Subcommand};
use pantools::cmd::convert;
use pantools::cmd::index;
use pantools::cmd::pav;
use pantools::error::CmdError;
use pantools::logging;
use pantools::resource;
use log;

const VERSION: &str = "version 0.1";

#[derive(Parser, Debug)]
#[command(
    author = "Zhang Yixing",
    version = VERSION,
    about = "pantools is a tool for converting GFA files",
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Subcli,
}

#[derive(Subcommand, Debug)]
#[allow(non_camel_case_types)]
enum Subcli {
    /// Convert GFA between GFA1.0 and GFA1.1
    convert {
        /// Input GFA file
        #[arg(short = 'g', long = "gfa", required = true)]
        input: String,

        /// Output GFA file
        #[arg(short = 'o', long = "output", required = true)]
        output: String,

        /// W2P flag <True 1 else False 0>
        #[arg(short = 'i', default_value = "1")]
        i: String,
    },
    /// Output PAV matrix of node list
    pav {
        /// Input GFA file
        #[arg(short = 'g', long = "gfa", required = true)]
        gfa: String,

        /// Input node list
        #[arg(short = 'n', long = "node", required = true)]
        node: String,

        /// Output PAV matrix
        #[arg(short = 'o', long = "output", required = true)]
        output: String,
    },
    /// Build index for GFA
    index {
        /// Input GFA file
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
    println!("{}", format!("Done!, pantools {}", VERSION));
    resource::gather_resources();
    Ok(())
}
