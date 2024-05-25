use crate::error::CmdError;
use crate::gfa;
use bstr::io::BufReadExt;
use bstr::ByteSlice;
use log;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::{fs::File, io::BufReader};

fn p2w(input: &[u8]) -> Vec<u8> {
    input
        .split(|&b| b == b',')
        .flat_map(|part| {
            let (direction, slice) = match part.ends_with(&[b'+']) {
                true => (b'>', &part[..part.len() - 1]),
                false => (b'<', &part[..part.len() - 1]),
            };
            std::iter::once(direction)
                .chain(slice.iter().copied())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn w2p(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < input.len() {
        let sign = input[i];
        let suffix = if sign == b'>' { b'+' } else { b'-' };
        i += 1;
        while i < input.len() && input[i].is_ascii_digit() {
            result.push(input[i]);
            i += 1;
        }
        result.push(suffix);
        if i < input.len() && (input[i] == b'>' || input[i] == b'<') {
            result.push(b',');
        }
    }
    result
}

fn write_with_error_handling<W: Write>(writer: &mut W, data: &[u8]) -> Result<(), CmdError> {
    writer.write_all(data).map_err(|_| CmdError::WriteError)
}

pub fn convert_1_1(path: String, output: String) -> Result<(), CmdError> {
    log::info!("Converting from 1.0 to 1.1");
    let file = File::open(&path).map_err(CmdError::FileOpenError)?;
    let lines = BufReader::new(file).byte_lines();
    let file = File::create(&output).map_err(|_| CmdError::CreateFileError)?;
    let mut output = BufWriter::new(file);
    let mut all_segment: HashMap<usize, usize> = HashMap::new();

    for line in lines {
        let line = line.map_err(CmdError::LineReadError)?;
        let l = line.as_bytes();

        if l.starts_with(b"H") {
            handle_header_line(&line, &mut output)?;
        } else if l.starts_with(b"P") {
            handle_p_line(&line, &mut output, &mut all_segment)?;
        } else {
            if l.starts_with(b"S") {
                handle_s_line(&line, &mut all_segment)?;
            }
            write_with_error_handling(&mut output, l)?;
            write_with_error_handling(&mut output, b"\n")?;
        }
    }
    output.flush().map_err(|_| CmdError::WriteError)?;
    Ok(())
}

fn handle_header_line(line: &Vec<u8>, output: &mut BufWriter<File>) -> Result<(), CmdError> {
    let new_line = line.replace(b"Z:1.0", b"Z:1.1");
    output
        .write_all(new_line.as_bytes())
        .map_err(|_| CmdError::WriteError)?;
    output.write_all(b"\n").map_err(|_| CmdError::WriteError)?;
    Ok(())
}

fn handle_p_line(
    line: &Vec<u8>,
    output: &mut BufWriter<File>,
    all_segment: &mut HashMap<usize, usize>,
) -> Result<(), CmdError> {
    let l = line.as_bytes();
    let parts: Vec<&[u8]> = l.split(|&b| b == b'\t').collect();
    let tem: Vec<&[u8]> = parts[1].split(|&b| b == b'#').collect();
    let chrom_parts: Vec<&[u8]> = tem[2].split(|&b| b == b':').collect();
    let (start, mut end) = if chrom_parts.len() > 1 {
        let range_parts: Vec<&[u8]> = chrom_parts[1].split(|&b| b == b'-').collect();
        let start = gfa::u8_slice_to_usize(range_parts[0])?;
        let end = gfa::u8_slice_to_usize(range_parts[1])?;
        (start, end)
    } else {
        (0, 0)
    };

    let new_w = p2w(parts[2]);

    if end == 0 {
        let end_vec: Vec<usize> = new_w
            .clone()
            .split(|&b| b == b'>' || b == b'<')
            .filter_map(|slice| gfa::u8_slice_to_usize(slice).ok())
            .collect();

        end = end_vec[1..]
            .iter()
            .filter_map(|&x| all_segment.get(&x))
            .map(|&x| x as usize)
            .sum();
    }
    write_with_error_handling(output, b"W\t")?;
    write_with_error_handling(output, tem[0])?;
    write_with_error_handling(output, b"\t")?;
    write_with_error_handling(output, tem[1])?;
    write_with_error_handling(output, b"\t")?;
    write_with_error_handling(output, chrom_parts[0])?;
    write_with_error_handling(output, b"\t")?;
    write_with_error_handling(output, start.to_string().as_bytes())?;
    write_with_error_handling(output, b"\t")?;
    write_with_error_handling(output, end.to_string().as_bytes())?;
    write_with_error_handling(output, b"\t")?;
    write_with_error_handling(output, &new_w)?;
    write_with_error_handling(output, b"\n")?;
    Ok(())
}

fn handle_s_line(line: &Vec<u8>, all_segment: &mut HashMap<usize, usize>) -> Result<(), CmdError> {
    let mut parts = line.split_str(b"\t");
    parts.next();
    let key = gfa::u8_slice_to_usize(parts.next().ok_or(CmdError::EmptyLine)?)?;
    let value = parts.next().ok_or(CmdError::EmptyLine)?.len();
    all_segment.insert(key, value);
    Ok(())
}

pub fn convert_1_0(path: String, output: String) -> Result<(), CmdError> {
    log::info!("Converting from 1.1 to 1.0");
    let file = File::open(&path).map_err(CmdError::FileOpenError)?;
    let lines = BufReader::new(file).byte_lines();
    let file = File::create(&output).map_err(|_| CmdError::CreateFileError)?;
    let mut output = BufWriter::new(file);

    for line in lines {
        let line = line.map_err(CmdError::LineReadError)?;
        let l = line.as_bytes();
        if l.starts_with(b"H") {
            handle_header_line_reverse(&line, &mut output)?;
        } else if l.starts_with(b"W") {
            handle_w_line(&line, &mut output)?;
        } else {
            write_with_error_handling(&mut output, l)?;
            write_with_error_handling(&mut output, b"\n")?;
        }
    }
    output.flush().map_err(|_| CmdError::WriteError)?;
    Ok(())
}

fn handle_header_line_reverse(
    line: &Vec<u8>,
    output: &mut BufWriter<File>,
) -> Result<(), CmdError> {
    let new_line = line.replace(b"Z:1.1", b"Z:1.0");
    output
        .write_all(new_line.as_bytes())
        .map_err(|_| CmdError::WriteError)?;
    output.write_all(b"\n").map_err(|_| CmdError::WriteError)?;
    Ok(())
}

fn handle_w_line(line: &Vec<u8>, output: &mut BufWriter<File>) -> Result<(), CmdError> {
    let parts: Vec<&[u8]> = line.as_bytes().split(|&b| b == b'\t').collect();
    write_with_error_handling(output, b"P\t")?;
    write_with_error_handling(output, parts[1])?;
    write_with_error_handling(output, b"#")?;
    write_with_error_handling(output, parts[2])?;
    write_with_error_handling(output, b"#")?;
    write_with_error_handling(output, parts[3])?;
    write_with_error_handling(output, b":")?;
    write_with_error_handling(output, parts[4])?;
    write_with_error_handling(output, b"-")?;
    write_with_error_handling(output, parts[5])?;
    write_with_error_handling(output, b"\t")?;
    write_with_error_handling(output, &w2p(parts[6]))?;
    write_with_error_handling(output, b"\n")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_p2w() {
        let input = b"11+,12-,13+";
        let expected = b">11<12>13";
        let result = p2w(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_w2p() {
        let input = b">11<12>13";
        let expected = b"11+,12-,13+";
        let result = w2p(input);
        assert_eq!(result, expected);
    }

    fn setup_test_file(data: &[u8], path: &str) {
        let mut file = File::create(path).expect("Unable to create test file");
        file.write_all(data)
            .expect("Unable to write data to test file");
    }

    fn read_test_file(path: &str) -> Vec<u8> {
        let mut file = File::open(path).expect("Unable to open test file");
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .expect("Unable to read data from test file");
        data
    }

    #[test]
    fn test_convert_1_1_case1() {
        let input_path = "test_input_1_0.gfa";
        let output_path = "test_output_1_1.gfa";
        let gfa_data = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            P\tsample#0#chr1:0-18\t11+,12-,13+\n";

        let expected = b"H\tVN:Z:1.1\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            W\tsample\t0\tchr1\t0\t18\t>11<12>13\n";

        setup_test_file(gfa_data, input_path);

        convert_1_1(input_path.to_string(), output_path.to_string()).expect("Conversion failed");

        let result = read_test_file(output_path);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_1_1_case2() {
        let input_path = "test_input_1_0_case2.gfa";
        let output_path = "test_output_1_1_case2.gfa";
        let gfa_data = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            P\tsample#0#chr1\t11+,12-,13+\t0M,0M\n";

        let expected = b"H\tVN:Z:1.1\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            W\tsample\t0\tchr1\t0\t18\t>11<12>13\n";

        setup_test_file(gfa_data, input_path);

        convert_1_1(input_path.to_string(), output_path.to_string()).expect("Conversion failed");

        let result = read_test_file(output_path);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_1_0() {
        let input_path = "test_input_1_1.gfa";
        let output_path = "test_output_1_0.gfa";
        let gfa_data = b"H\tVN:Z:1.1\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            W\tsample\t0\tchr1\t0\t18\t>11<12>13\n";

        let expected = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            P\tsample#0#chr1:0-18\t11+,12-,13+\n";

        setup_test_file(gfa_data, input_path);

        convert_1_0(input_path.to_string(), output_path.to_string()).expect("Conversion failed");

        let result = read_test_file(output_path);
        assert_eq!(result, expected);
    }
}
