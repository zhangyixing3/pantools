use bstr::{io::BufReadExt, ByteSlice};
use crate::error::CmdError;
use anyhow::Result;
use log::{debug, error, info, warn};
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

    pub fn build(&mut self) -> GFAParser{
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



impl GFAParser {
    /// Create a new GFAParser that will parse all four GFA line
    /// types, and use the optional fields parser and storage `T`.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse_gfa_line(&self, bytes: &[u8]) ->  {
        let line = bytes.trim_with(|c| c.is_ascii_whitespace());
        let mut fields = line.split_str(b"\t");
        let hdr = fields.next().ok_or(CmdError::EmptyLine)?;

        let entity = match hdr {
            b"H" => Header::parse_line(fields).map_err(CmdError::LineReadError),
            b"S" => if self.segments {
                Segment::parse_line(fields).map_err(CmdError::LineReadError)
            } else {
                return Ok(None)
            },
            b"L" => if self.links {
                Link::parse_line(fields).map_err(CmdError::LineReadError)
            } else {
                return Ok(None)
            },
            b"W" => if self.walks {
                Walk::parse_line(fields).map_err(CmdError::LineReadError)
            } else {
                return Ok(None)
            },
            b"P" => if self.paths {
                Path::parse_line(fields).map_err(CmdError::LineReadError)
            } else {
                return Ok(None)
            },
            _ => return Err(CmdError::UnknownLineType),
        }?;

        Ok(entity)
    }



    pub fn parse_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) ->  Result<GFA, CmdError>{
        use std::{fs::File, io::BufReader};

        let file = File::open(path).map_err(CmdError::FileOpenError)?;
        let lines = BufReader::new(file).byte_lines();

        let mut gfa = GFA::new();

        for line in lines {
            let line = line.map_err(CmdError::LineReadError)?;
            match self.parse_gfa_line(line.as_ref()) {
            };
        }

        Ok(gfa)
    }
}


pub struct GFA {
    pub headers: Option<Header>,
    pub segments: Option<Vec<Segment>>,
    pub links: Option<Vec<Link>>,
    pub walks: Option<Vec<Walk>>,
    pub paths: Option<Vec<Path>>,
}

impl GFA {
    pub fn new() -> Self {
        GFA {
            headers: None,
            segments: Some(Vec::new()),
            links: Some(Vec::new()),
            walks: Some(Vec::new()),
            paths: Some(Vec::new()),
        }
    }
        }

pub struct Header {
    pub version: String,
    pub samples: Option<Vec<String>>,
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

pub struct Range {
    pub start: u32,
    pub end: u32,
}
trait GfaParsable {
    fn parse_line<'a>(fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> where Self: Sized;
}

impl GfaParsable for Header {
    fn parse_line<'a>(fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let mut fields_iter = fields;
        fields_iter.next();
        let version_field = fields_iter.next().ok_or(CmdError::EmptyLine)?;
        let version = if version_field.starts_with(b"VN:Z:") {
            String::from_utf8_lossy(&version_field[5..]).into_owned()
        } else {
            return Err(CmdError::EmptyLine)
        };

        let mut samples: Vec<String> = vec![];

        for field in fields_iter {
            let sample_info: String;
            if field.starts_with(b"RS:Z:") {
                sample_info = String::from_utf8_lossy(&field[5..]).into_owned();
            }else {
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
    fn parse_line<'a>(fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let mut fields_iter = fields;
        fields_iter.next();
        let id = u8_slice_to_usize(fields_iter.next().ok_or(CmdError::EmptyLine)?)?;
        let sequence = fields_iter.next().ok_or_else(|| CmdError::LineReadError(std::io::Error::new(std::io::ErrorKind::Other, "No sequence found for segment.")))?.to_vec();
        Ok(Segment { id, sequence })
    }
}

impl GfaParsable for Link {
    fn parse_line<'a>(fields: impl Iterator<Item = &'a [u8]>) -> Result<Self, CmdError> {
        let mut fields_iter = fields;
        fields_iter.next();
        let from_segment: usize = u8_slice_to_usize(fields_iter.next().ok_or(CmdError::EmptyLine)?)?;
        let tem = fields_iter.next().ok_or(CmdError::EmptyLine)?;
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
        let to_segment: usize = u8_slice_to_usize(fields_iter.next().ok_or(CmdError::EmptyLine)?)?;
        let tem = fields_iter.next().ok_or(CmdError::EmptyLine)?;
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
        fields.next();
        let sample  = String::from_utf8_lossy(fields.next().ok_or(CmdError::EmptyLine)?).into_owned();
        let haptype : u8 = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)? as u8;
        let chr : String = String::from_utf8_lossy(fields.next().ok_or(CmdError::EmptyLine)?).into_owned();
        let start : u32 = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)? as u32;
        let end : u32 = u8_slice_to_usize(fields.next().ok_or(CmdError::EmptyLine)?)? as u32;
        let unit : Vec<u8> = fields.next().ok_or(CmdError::EmptyLine)?.to_vec();
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
        fields.next();
        let line_str = String::from_utf8_lossy(fields.next().ok_or(CmdError::EmptyLine)?);
        let parts: Vec<&str> = line_str.split("#").collect();

        if parts.len() < 3 {
            return Err(CmdError::ParseError);
        }

        let sample = parts[0].to_owned();

        let haptype = parts[1].parse::<u8>()
        .map_err(|_| CmdError::ParseError)?;

        let chroms_info = parts[2].split(":").collect::<Vec<&str>>();
        if chroms_info.len() < 2 {
            log::debug!("The Path line has no range information.");
        }

        let chroms = chroms_info[0].to_owned();

        let range_info = chroms_info[1].split("-").collect::<Vec<&str>>();
        let ranges = if range_info.len() == 2 {
            let start = range_info[0].parse::<u32>().map_err(|_| CmdError::ParseError)?;
            let end = range_info[1].parse::<u32>().map_err(|_| CmdError::ParseError)?;
            Some(Range { start, end })
        } else {
            None
        };
        let unit = fields.next().ok_or(CmdError::EmptyLine)?.to_vec();

        Ok(Path { sample, haptype, chroms, ranges,unit })
    }
}

