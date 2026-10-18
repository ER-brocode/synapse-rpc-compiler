use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SynapseError {
    #[error("Lexer error at line {line}, column {col}: {message}")]
    LexerError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Parse error at line {line}, column {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        context: Option<String>,
    },

    #[error("Codec error: {message}")]
    CodecError {
        message: String,
    },

    #[error("Protocol error: {message}")]
    ProtocolError {
        message: String,
    },

    #[error("Multiplexer error: {message}")]
    MultiplexerError {
        message: String,
    },

    #[error("Codegen error for target '{target}': {message}")]
    CodegenError {
        target: String,
        message: String,
    },

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for SynapseError {
    fn from(err: std::io::Error) -> Self {
        SynapseError::IoError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SynapseError>;
