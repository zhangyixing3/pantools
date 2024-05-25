use crate::{error::CmdError, gfa};
use std::io::Write;
use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
};

use log;
pub fn run(gfa: String, node: String, output: String) -> Result<(), CmdError> {
    let gfa_parser = gfa::GFAParserBuilder::new().get_walks(true).build();
    let gfa_obj = gfa_parser.parse_file(gfa)?;
    log::debug!("GFA file parsed successfully");
    let reader = BufReader::new(std::fs::File::open(node).map_err(CmdError::FileOpenError)?);
    let mut avec = HashSet::new();
    for line in reader.lines() {
        let line = line.map_err(CmdError::LineReadError)?;
        let line = line.trim();
        avec.insert(line.parse::<usize>().map_err(|_| CmdError::ParseError)?);
    }
    log::debug!("The number of nodes to be analyzed is: {}", avec.len());
    let mut samples: HashSet<String> = HashSet::new();
    for walk in &gfa_obj.walks {
        samples.insert(walk.sample.clone());
    }
    let mut matrix: HashMap<&String, HashMap<usize, u32>> = HashMap::with_capacity(samples.len());
    for sample in &samples {
        matrix.insert(sample, HashMap::with_capacity(avec.len()));
    }
    log::debug!("total number of samples: {}", samples.len());
    for walk in gfa_obj.walks {
        for i in walk.extract_node() {
            for &ii in &avec {
                if i == ii {
                    if let Some(m) = matrix.get_mut(&walk.sample) {
                        if let Some(v) = m.get_mut(&i) {
                            *v += 1;
                        } else {
                            m.insert(i, 1);
                        }
                    }
                }
            }
        }
    }
    let mut writer = std::fs::File::create(output).map_err(CmdError::FileOpenError)?;
    let mut header = Vec::new();
    header.push("node".to_string());
    for sample in &samples {
        header.push(sample.to_owned());
    }
    writeln!(&mut writer, "{}", header.join("\t")).map_err(|_| CmdError::WriteError)?;
    for i in avec {
        let mut tem_vec = Vec::new();
        for j in &samples {
            if let Some(m) = matrix.get(j) {
                if let Some(v) = m.get(&i) {
                    tem_vec.push(*v);
                } else {
                    tem_vec.push(0);
                }
            }
        }
        let tem: Vec<String> = tem_vec.iter().map(|x| x.to_string()).collect();
        writeln!(&mut writer, "{}\t{}", i, tem.join("\t")).map_err(|_| CmdError::WriteError)?;
    }
    writer.flush().map_err(|_| CmdError::WriteError)?;

    Ok(())
}
