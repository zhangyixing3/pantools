// src/other_module.rs
use log::{debug, error, info, warn};
mod logging;
mod resource;
pub mod gfa;
mod error;

fn main() {
    logging::init_logging();
    debug!("Main|This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
    resource::gather_resources();
}
