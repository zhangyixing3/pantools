// src/other_module.rs
use log::{debug, error, info, warn};
mod error;
pub mod gfa;
mod logging;
mod resource;

fn main() {
    logging::init_logging();
    debug!("Main|This is a debug message");
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
    resource::gather_resources();
}
