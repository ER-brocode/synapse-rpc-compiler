use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub offset: usize,
}

impl Span {
    pub fn new(line: usize, col: usize, offset: usize) -> Self {
        Self { line, col, offset }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Namespace,
    Import,
    Struct,
    Message,
    Enum,
    Service,
    Rpc,
    Stream,
    PubSub,
    Subscribe,
    Deprecated,
    Required,
    Optional,

    // Primitive Type Keywords
    TypeBool,
    TypeU8,
    TypeU16,
    TypeU32,
    TypeU64,
    TypeI8,
    TypeI16,
    TypeI32,
    TypeI64,
    TypeF32,
    TypeF64,
    TypeString,
    TypeBytes,
    TypeUuid,
    TypeTimestamp,

    // Container / generic type keywords
    TypeList,
    TypeMap,
    TypeOption,
    TypeResult,

    // Identifiers and Literals
    Ident(String),
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    DocComment(String),

    // Symbols & Operators
    LBrace,        // {
    RBrace,        // }
    LParen,        // (
    RParen,        // )
    LBracket,      // [
    RBracket,      // ]
    LAngle,        // <
    RAngle,        // >
    Colon,         // :
    Semicolon,     // ;
    Comma,         // ,
    Equals,        // =
    Arrow,         // ->
    At,            // @
    Dot,           // .

    // End of stream
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Namespace => write!(f, "namespace"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Message => write!(f, "message"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Service => write!(f, "service"),
            TokenKind::Rpc => write!(f, "rpc"),
            TokenKind::Stream => write!(f, "stream"),
            TokenKind::PubSub => write!(f, "pubsub"),
            TokenKind::Subscribe => write!(f, "subscribe"),
            TokenKind::Deprecated => write!(f, "deprecated"),
            TokenKind::Required => write!(f, "required"),
            TokenKind::Optional => write!(f, "optional"),
            TokenKind::TypeBool => write!(f, "bool"),
            TokenKind::TypeU8 => write!(f, "u8"),
            TokenKind::TypeU16 => write!(f, "u16"),
            TokenKind::TypeU32 => write!(f, "u32"),
            TokenKind::TypeU64 => write!(f, "u64"),
            TokenKind::TypeI8 => write!(f, "i8"),
            TokenKind::TypeI16 => write!(f, "i16"),
            TokenKind::TypeI32 => write!(f, "i32"),
            TokenKind::TypeI64 => write!(f, "i64"),
            TokenKind::TypeF32 => write!(f, "f32"),
            TokenKind::TypeF64 => write!(f, "f64"),
            TokenKind::TypeString => write!(f, "string"),
            TokenKind::TypeBytes => write!(f, "bytes"),
            TokenKind::TypeUuid => write!(f, "uuid"),
            TokenKind::TypeTimestamp => write!(f, "timestamp"),
            TokenKind::TypeList => write!(f, "list"),
            TokenKind::TypeMap => write!(f, "map"),
            TokenKind::TypeOption => write!(f, "option"),
            TokenKind::TypeResult => write!(f, "result"),
            TokenKind::Ident(s) => write!(f, "identifier '{}'", s),
            TokenKind::StringLiteral(s) => write!(f, "string \"{}\"", s),
            TokenKind::IntLiteral(n) => write!(f, "integer {}", n),
            TokenKind::FloatLiteral(v) => write!(f, "float {}", v),
            TokenKind::BoolLiteral(b) => write!(f, "boolean {}", b),
            TokenKind::DocComment(c) => write!(f, "doc comment \"{}\"", c),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LAngle => write!(f, "<"),
            TokenKind::RAngle => write!(f, ">"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Eof => write!(f, "<EOF>"),
        }
    }
}
