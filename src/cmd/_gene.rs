use std::{fs::File, io::{BufRead, BufReader}};

use crate::error::CmdError;
use bstr::io::BufReadExt;
use log;
pub fn gene_gfa(gene:String, pafs:String, out:String) -> Result<(),CmdError> {
    log::info!("loading gene data file");
    let gene_f = File::open(gene).map_err( CmdError::FileOpenError)?;
    let lines = BufReader::new(gene_f).byte_lines();
    let mut gene_l = std::collections::HashMap::new();
    let mut begin:u32 = 0;
    for line in lines {
        let line = line.map_err( CmdError::LineReadError)?;
        if line.starts_with(b">") {
            let gene_name = &line[1..];
            gene_l.insert(gene_name.to_vec(), begin);
            begin  += 1;
        }
    }
    log::info!("Total gene number: {}", gene_l.len());
    log::info!("loading paf data file");
    let pafs_f = File::open(pafs).map_err( CmdError::FileOpenError)?;
    for paf in BufReader::new(pafs_f).lines() {
        let paf = paf.map_err( CmdError::LineReadError)?;
    }


    let paf_f = File::open(paf).map_err( CmdError::FileOpenError)?;
    let lines = BufReader::new(paf_f).byte_lines();
    for line in lines {
        let line = line.map_err( CmdError::LineReadError)?;
        let fields:Vec<&str> = line.split('\t').collect();
        let qname = fields[0];
        let tname = fields[5];
        let qstart = fields[7].parse::<u32>().map_err( CmdError::ParseIntError)?;
        let qend = fields[8].parse::<u32>().map_err( CmdError::ParseIntError)?;
    }


    Ok(())
}
