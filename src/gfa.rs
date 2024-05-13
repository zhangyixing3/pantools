use crate::error::CmdError;
use anyhow::Result;
use bstr::{io::BufReadExt, ByteSlice};
use std::collections::HashMap;

/// Builder struct for GFAParsers
#[derive(Debug, Default, Clone, Copy)]
pub struct GFAParserBuilder {
    pub segments: bool,
    pub links: bool,
    pub walks: bool,
    pub paths: bool,
}

impl GFAParserBuilder {
    /// Parse no GFA lines, useful if you only want to parse one line type.
    pub fn new() -> Self {
        GFAParserBuilder {
            segments: false,
            links: false,
            walks: false,
            paths: false,
        }
    }

    /// Parse all GFA lines.
    pub fn all() -> Self {
        GFAParserBuilder {
            segments: true,
            links: true,
            walks: true,
            paths: true,
        }
    }

    pub fn get_segments(&mut self, include: bool) -> &mut Self {
        self.segments = include;
        self
    }

    pub fn get_links(&mut self, include: bool) -> &mut Self {
        self.links = include;
        self
    }

    pub fn get_paths(&mut self, include: bool) -> &mut Self {
        self.paths = include;
        self
    }
    pub fn get_walks(&mut self, include: bool) -> &mut Self {
        self.walks = include;
        self
    }

    pub fn build(&mut self) -> GFAParser {
        GFAParser {
            segments: self.segments,
            links: self.links,
            walks: self.walks,
            paths: self.paths,
        }
    }
}
#[derive(Debug, Clone)]
pub struct GFAParser {
    segments: bool,
    links: bool,
    walks: bool,
    paths: bool,
}

impl Default for GFAParser {
    fn default() -> Self {
        let mut config = GFAParserBuilder::all();
        config.build()
    }
}

pub enum GfaEntity {
    Header(Header),
    Segment(Segment),
    Link(Link),
    Walk(Walk),
    Path(Path),
}

impl GFAParser {
    /// Create a new GFAParser that will parse all four GFA line
    /// types, and use the optional fields parser and storage `T`.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_gfa_line(&self, bytes: &[u8]) -> Result<Option<GfaEntity>, CmdError> {
        let line = bytes.trim_with(|c| c.is_ascii_whitespace());
        let mut fields = line.split_str(b"\t");
        let hdr = fields.next().ok_or(CmdError::EmptyLine)?;

        match hdr {
            b"H" => Ok(Some(GfaEntity::Header(Header::parse_line(fields)?))),
            b"S" => {
                if self.segments {
                    Ok(Some(GfaEntity::Segment(Segment::parse_line(fields)?)))
                } else {
                    Ok(None)
                }
            }
            b"L" => {
                if self.links {
                    Ok(Some(GfaEntity::Link(Link::parse_line(fields)?)))
                } else {
                    Ok(None)
                }
            }
            b"W" => {
                if self.walks {
                    Ok(Some(GfaEntity::Walk(Walk::parse_line(fields)?)))
                } else {
                    Ok(None)
                }
            }
            b"P" => {
                if self.paths {
                    Ok(Some(GfaEntity::Path(Path::parse_line(fields)?)))
                } else {
                    Ok(None)
                }
            }
            _ => Err(CmdError::UnknownLineType),
        }
    }

    pub fn parse_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<GFA, CmdError> {
        use std::{fs::File, io::BufReader};

        let file = File::open(path).map_err(CmdError::FileOpenError)?;
        let lines = BufReader::new(file).byte_lines();

        let mut gfa = GFA::new();

        for line in lines {
            let line = line.map_err(CmdError::LineReadError)?;
            if let Some(entity) = self.parse_gfa_line(line.as_bytes())? {
                gfa.add_entity(entity);
            }
        }

        Ok(gfa)
    }
}

pub struct GFA {
    pub headers: Header,
    pub segments: Vec<Segment>,
    pub links: Vec<Link>,
    pub walks: Vec<Walk>,
    pub paths: Vec<Path>,
}

impl GFA {
    pub fn new() -> Self {
        GFA {
            headers: Header::new(), // Initialize empty vectors
            segments: Vec::new(),
            links: Vec::new(),
            walks: Vec::new(),
            paths: Vec::new(),
        }
    }
    pub fn add_entity(&mut self, entity: GfaEntity) {
        match entity {
            GfaEntity::Header(header) => self.headers = header,
            GfaEntity::Segment(segment) => self.segments.push(segment),
            GfaEntity::Link(link) => self.links.push(link),
            GfaEntity::Walk(walk) => self.walks.push(walk),
            GfaEntity::Path(path) => self.paths.push(path),
        }
    }
    pub fn get_segment_len(&self) -> HashMap<usize, usize> {
        let mut len_map: HashMap<usize, usize> = HashMap::with_capacity(self.segments.len());
        for segment in self.segments.iter() {
            len_map.insert(segment.id, segment.sequence.len());
        }
        len_map
    }
}

pub struct Header {
    pub version: String,
    pub samples: Option<Vec<String>>,
}
impl Header {
    fn new() -> Self {
        Header {
            version: String::new(),
            samples: None,
        }
    }
}

pub struct Segment {
    pub id: usize,
    pub sequence: Vec<u8>,
}
pub struct Link {
    pub from_segment: usize,
    pub from_orient: bool,
    pub to_segment: usize,
    pub to_orient: bool,
}
pub struct Path {
    pub sample: String,
    pub haptype: u8,
    pub chroms: String,
    pub ranges: Option<Range>,
    pub unit: Vec<u8>,
}
pub struct Walk {
    pub sample: String,
    pub haptype: u8,
    pub chroms: String,
    pub ranges: Range,
    pub unit: Vec<u8>,
}
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}
trait GfaParsable {
    fn parse_line<'a>(fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError>
    where
        Self: Sized;
}

impl GfaParsable for Header {
    fn parse_line<'a>(mut fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let version_field = fields.next().ok_or(CmdError::EmptyLine)?;
        let version = if version_field.starts_with(b"VN:Z:") {
            String::from_utf8_lossy(&version_field[5..]).into_owned()
        } else {
            return Err(CmdError::EmptyLine);
        };

        let mut samples: Vec<String> = vec![];

        for field in fields {
            let sample_info: String;
            if field.starts_with(b"RS:Z:") {
                sample_info = String::from_utf8_lossy(&field[5..]).into_owned();
            } else {
                sample_info = String::from_utf8_lossy(&field).into_owned();
            }
            samples.push(sample_info);
        }
        let samples = if samples.is_empty() {
            None
        } else {
            Some(samples)
        };

        Ok(Header { version, samples })
    }
}
fn u8_slice_to_usize(slice: &[u8]) -> Result<usize, CmdError> {
    let mut num = 0;

    for &b in slice {
        num = num * 10 + (b - b'0') as usize;
    }

    Ok(num)
}
impl GfaParsable for Segment {
    fn parse_line<'a>(mut fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let id = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)?;
        let sequence = fields
            .next()
            .ok_or_else(|| {
                CmdError::LineReadError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No sequence found for segment.",
                ))
            })?
            .to_vec();
        Ok(Segment { id, sequence })
    }
}

impl GfaParsable for Link {
    fn parse_line<'a>(mut fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let from_segment: usize = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)?;
        let tem = fields.next().ok_or(CmdError::EmptyLine)?;
        let from_orient: bool;
        match tem {
            b"+" => {
                from_orient = true;
            }
            b"-" => {
                from_orient = false;
            }
            _ => {
                return Err(CmdError::ParseError);
            }
        }
        let to_segment: usize = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)?;
        let tem = fields.next().ok_or(CmdError::EmptyLine)?;
        let to_orient: bool;
        match tem {
            b"+" => {
                to_orient = true;
            }
            b"-" => {
                to_orient = false;
            }
            _ => {
                return Err(CmdError::ParseError);
            }
        }
        Ok(Link {
            from_segment,
            from_orient,
            to_segment,
            to_orient,
        })
    }
}
impl GfaParsable for Walk {
    fn parse_line<'a>(mut fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let sample =
            String::from_utf8_lossy(fields.next().ok_or(CmdError::EmptyLine)?).into_owned();
        let haptype: u8 = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)? as u8;
        let chr: String =
            String::from_utf8_lossy(fields.next().ok_or(CmdError::EmptyLine)?).into_owned();
        let start: u32 = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)? as u32;
        let end: u32 = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)? as u32;
        let unit: Vec<u8> = fields.next().ok_or(CmdError::EmptyLine)?.to_vec();
        Ok(Walk {
            sample,
            haptype,
            chroms: chr,
            ranges: Range { start, end },
            unit,
        })
    }
}

impl GfaParsable for Path {
    fn parse_line<'a>(mut fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let line_str = String::from_utf8_lossy(fields.next().ok_or(CmdError::EmptyLine)?);
        let parts: Vec<&str> = line_str.split("#").collect();

        if parts.len() < 3 {
            return Err(CmdError::ParseError);
        }

        let sample = parts[0].to_owned();

        let haptype = parts[1].parse::<u8>().map_err(|_| CmdError::ParseError)?;

        let chroms_info = parts[2].split(":").collect::<Vec<&str>>();
        let chroms = chroms_info[0].to_owned();
        let ranges = if chroms_info.len() == 2 {
            let range_info = chroms_info[1].split("-").collect::<Vec<&str>>();

            let start = range_info[0]
                .parse::<u32>()
                .map_err(|_| CmdError::ParseError)?;
            let end = range_info[1]
                .parse::<u32>()
                .map_err(|_| CmdError::ParseError)?;
            Some(Range { start, end })
        } else {
            None
        };

        let unit = fields.next().ok_or(CmdError::EmptyLine)?.to_vec();

        Ok(Path {
            sample,
            haptype,
            chroms,
            ranges,
            unit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::BufReader;
    use std::io::Cursor;

    #[test]
    fn test_parse_p_gfa1() {
        let gfa_data = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t4M\n\
            L\t12\t-\t13\t+\t5M\n\
            L\t11\t+\t13\t+\t3M\n\
            P\t14#0#chr1\t11+,12-,13+\t4M,5M";
        let cursor = Cursor::new(gfa_data);
        let reader = BufReader::new(cursor);

        let parser = GFAParser::default();
        let mut gfa = GFA::new();

        for line in reader.byte_lines() {
            let line = line.expect("Failed to read line");

            if let Some(entity) = parser
                .parse_gfa_line(&line)
                .expect("Failed to parse GFA line")
            {
                gfa.add_entity(entity);
            }
        }

        // Assertions to verify the content of GFA structure
        assert_eq!(gfa.headers.version, "1.0");
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 11 && s.sequence == b"ACCTT".as_ref()));
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 12 && s.sequence == b"TCAAGG".as_ref()));
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 13 && s.sequence == b"CTTGATT".as_ref()));
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 11 && l.to_segment == 12));
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 12 && l.to_segment == 13));
        assert!(gfa.paths.iter().any(|p| p.sample == "14"));
    }
    #[test]
    fn test_parse_p_gfa2() {
        let gfa_data = b"H\tVN:Z:1.0\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            P\t14#0#chr1:0-36\t11+,12-,13+";
        let cursor = Cursor::new(gfa_data);
        let reader = BufReader::new(cursor);

        let parser = GFAParser::default();
        let mut gfa = GFA::new();

        for line in reader.byte_lines() {
            let line = line.expect("Failed to read line");
            if let Some(entity) = parser
                .parse_gfa_line(&line)
                .expect("Failed to parse GFA line")
            {
                gfa.add_entity(entity);
            }
        }

        // Assertions to verify the content of GFA structure
        assert_eq!(gfa.headers.version, "1.0");
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 11 && s.sequence == b"ACCTT".as_ref()));
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 12 && s.sequence == b"TCAAGG".as_ref()));
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 13 && s.sequence == b"CTTGATT".as_ref()));

        // Check links
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 11 && l.to_segment == 12));
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 12 && l.to_segment == 13));
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 11 && l.to_segment == 13));

        // Check path
        assert_eq!(gfa.paths[0].sample, "14");
        assert_eq!(gfa.paths[0].chroms, "chr1");

        // More detailed check for the ranges
        assert!(gfa.paths[0].ranges.is_some());
        let ranges = gfa.paths[0].ranges.as_ref().unwrap();
        assert_eq!(ranges.start, 0, "The start of the range should be 0");
        assert_eq!(ranges.end, 36, "The end of the range should be 36");
    }
    #[test]
    fn test_parse_w_gfa() {
        let gfa_data = b"H\tVN:Z:1.1\n\
            S\t11\tACCTT\n\
            S\t12\tTCAAGG\n\
            S\t13\tCTTGATT\n\
            L\t11\t+\t12\t-\t0M\n\
            L\t12\t-\t13\t+\t0M\n\
            L\t11\t+\t13\t+\t0M\n\
            W\tsample\t0\tchr1\t0\t36\t>11<12>13";
        let cursor = Cursor::new(gfa_data);
        let reader = BufReader::new(cursor);

        let parser = GFAParser::default();
        let mut gfa = GFA::new();

        for line in reader.byte_lines() {
            let line = line.expect("Failed to read line");
            if let Some(entity) = parser
                .parse_gfa_line(&line)
                .expect("Failed to parse GFA line")
            {
                gfa.add_entity(entity);
            }
        }

        // Assertions to verify the content of GFA structure
        assert_eq!(gfa.headers.version, "1.1");
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 11 && s.sequence == b"ACCTT".as_ref()));
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 12 && s.sequence == b"TCAAGG".as_ref()));
        assert!(gfa
            .segments
            .iter()
            .any(|s| s.id == 13 && s.sequence == b"CTTGATT".as_ref()));

        // Check links
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 11 && l.to_segment == 12));
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 12 && l.to_segment == 13));
        assert!(gfa
            .links
            .iter()
            .any(|l| l.from_segment == 11 && l.to_segment == 13));

        // Check walk
        assert_eq!(gfa.walks[0].sample, "sample");
        assert_eq!(gfa.walks[0].chroms, "chr1");

        let ranges = gfa.walks[0].ranges;
        assert_eq!(ranges.start, 0, "The start of the range should be 0");
        assert_eq!(ranges.end, 36, "The end of the range should be 36");
    }
}
