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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn test_pav() {
        let temp_dir = TempDir::new("test_pav").unwrap();
        let gfa_file_path = temp_dir.path().join("test.gfa");
        let node_file_path = temp_dir.path().join("test.nodes");
        let output_file_path = temp_dir.path().join("output.tsv");

        // create a sample GFA file
        let gfa_data = b"H\tVN:Z:1.1\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            W\tsample1\t0\tchr1\t0\t18\t>11<12>13\n\
            W\tsample2\t0\tchr1\t0\t11\t>11<12\n\
            W\tsample3\t0\tchr1\t0\t12\t>11<13\n\
            W\tsample4\t0\tchr1\t0\t10\t>11<11\n\
            W\tsample5\t0\tchr1\t0\t11\t>12>11\n";
        let mut gfa_file = File::create(&gfa_file_path).unwrap();
        gfa_file.write_all(gfa_data).unwrap();

        // create a node list file
        let mut node_file = File::create(&node_file_path).unwrap();
        writeln!(node_file, "11").unwrap();
        writeln!(node_file, "12").unwrap();
        writeln!(node_file, "13").unwrap();

        let gfa_file_str = gfa_file_path.to_str().unwrap().to_string();
        let node_file_str = node_file_path.to_str().unwrap().to_string();
        let output_file_str = output_file_path.to_str().unwrap().to_string();

        let result = run(gfa_file_str, node_file_str, output_file_str.clone());

        assert!(result.is_ok());
        let output_content = std::fs::read_to_string(output_file_path).unwrap();
        let expected_content = "node\tsample1\tsample2\tsample3\tsample4\
            \tsample5\n11\t1\t1\t1\t2\t1\n12\t1\t1\t0\
            \t0\t1\n13\t1\t0\t1\t0\t0\n";

        let output_lines: Vec<String> = output_content
            .trim()
            .split('\n')
            .map(String::from)
            .collect();
        let expected_lines: Vec<String> = expected_content
            .trim()
            .split('\n')
            .map(String::from)
            .collect();

        let mut output_columns: Vec<String> = output_lines
            .iter()
            .map(|line| {
                let mut columns: Vec<&str> = line.split('\t').collect();
                columns.sort();
                columns.join("\t")
            })
            .collect();

        let mut expected_columns: Vec<String> = expected_lines
            .iter()
            .map(|line| {
                let mut columns: Vec<&str> = line.split('\t').collect();
                columns.sort();
                columns.join("\t")
            })
            .collect();

        output_columns.sort();
        expected_columns.sort();

        assert_eq!(output_columns, expected_columns);
    }
}
