use crate::{error::CmdError, gfa};
use log;

pub fn prune_gfa(gfa: String, output: String) -> Result<(), CmdError> {
    let gfa_parser = gfa::GFAParserBuilder::new()
        .get_walks(true)
        .get_segments(true)
        .build();
    let gfa_obj = gfa_parser.parse_file(gfa)?;
    log::info!("GFA file parsed successfully");
    let walk_num = gfa_obj.walks.len();
    log::info!("Total number of walks: {}", walk_num);
    let segment_map = gfa_obj.get_segment_len();

    Ok(())
}
