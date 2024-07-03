use crate::error::CmdError;
use bstr::io::BufReadExt;
use bstr::ByteSlice;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

pub fn build(gfa: &str) -> Result<(), CmdError> {
    // Create output file
    let mut gfa_simple =
        BufWriter::new(File::create(format!("{}.simple", gfa)).map_err(CmdError::FileOpenError)?);

    // Open input file
    let file = File::open(gfa).map_err(CmdError::FileOpenError)?;
    let reader = BufReader::new(file);

    // Process each line in the input file
    for line in reader.byte_lines() {
        let line = line.map_err(CmdError::LineReadError)?;

        match line.get(0) {
            Some(&b'P') | Some(&b'W') => {
                // Split the line into parts by tabs
                let mut parts: Vec<&[u8]> = line.split(|&b| b == b'\t').collect();
                let node_index = if line[0] == b'P' { 2 } else { 6 };
                parts[node_index] = b"*";

                let joined_line = parts.join(&b"\t"[..]);

                gfa_simple
                    .write_all(joined_line.as_bytes())
                    .map_err(|_| CmdError::WriteError)?;
                gfa_simple
                    .write_all(b"\n")
                    .map_err(|_| CmdError::WriteError)?;
            }
            // Skip lines that don't start with 'P' or 'W'
            _ => continue,
        }
    }

    // Flush buffer to ensure all data is written
    gfa_simple.flush().map_err(|_| CmdError::WriteError)?;

    Ok(())
}
