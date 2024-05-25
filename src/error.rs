use thiserror::Error;

#[derive(Error, Debug)]
pub enum CmdError {
    #[error("Failed to open file: {0}")]
    FileOpenError(std::io::Error),

    #[error("Failed to read line: {0}")]
    LineReadError(std::io::Error),

    #[error("Failed to parse line")]
    ParseError,

    #[error("The line is empty")]
    EmptyLine,

    #[error("write error")]
    WriteError,

    #[error("Failed to create file")]
    CreateFileError,

    #[error("Unknown line type")]
    UnknownLineType,
}
