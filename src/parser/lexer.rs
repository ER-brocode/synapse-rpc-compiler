use crate::error::{Result, SynapseError};
use crate::parser::token::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            let start_line = self.line;
            let start_col = self.col;
            let start_offset = self.current_offset();

            let span = Span::new(start_line, start_col, start_offset);

            // Check for comments
            if self.peek_char() == '/' {
                if self.peek_ahead(1) == Some('/') {
                    // Check if it's a doc comment ///
                    if self.peek_ahead(2) == Some('/') {
                        let doc = self.read_line_doc_comment();
                        tokens.push(Token::new(TokenKind::DocComment(doc), span));
                        continue;
                    } else {
                        self.skip_line_comment();
                        continue;
                    }
                } else if self.peek_ahead(1) == Some('*') {
                    self.skip_block_comment()?;
                    continue;
                }
            }

            let ch = self.peek_char();

            let token = match ch {
                '{' => { self.advance(); Token::new(TokenKind::LBrace, span) }
                '}' => { self.advance(); Token::new(TokenKind::RBrace, span) }
                '(' => { self.advance(); Token::new(TokenKind::LParen, span) }
                ')' => { self.advance(); Token::new(TokenKind::RParen, span) }
                '[' => { self.advance(); Token::new(TokenKind::LBracket, span) }
                ']' => { self.advance(); Token::new(TokenKind::RBracket, span) }
                '<' => { self.advance(); Token::new(TokenKind::LAngle, span) }
                '>' => { self.advance(); Token::new(TokenKind::RAngle, span) }
                ':' => { self.advance(); Token::new(TokenKind::Colon, span) }
                ';' => { self.advance(); Token::new(TokenKind::Semicolon, span) }
                ',' => { self.advance(); Token::new(TokenKind::Comma, span) }
                '=' => { self.advance(); Token::new(TokenKind::Equals, span) }
                '@' => { self.advance(); Token::new(TokenKind::At, span) }
                '.' => { self.advance(); Token::new(TokenKind::Dot, span) }
                '-' => {
                    if self.peek_ahead(1) == Some('>') {
                        self.advance();
                        self.advance();
                        Token::new(TokenKind::Arrow, span)
                    } else if self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
                        self.read_number(span)?
                    } else {
                        return Err(SynapseError::LexerError {
                            line: self.line,
                            col: self.col,
                            message: format!("Unexpected character '-'"),
                        });
                    }
                }
                '"' => self.read_string(span)?,
                c if c.is_ascii_digit() => self.read_number(span)?,
                c if c.is_ascii_alphabetic() || c == '_' => self.read_ident_or_keyword(span),
                other => {
                    return Err(SynapseError::LexerError {
                        line: self.line,
                        col: self.col,
                        message: format!("Unexpected character: '{}'", other),
                    });
                }
            };

            tokens.push(token);
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.line, self.col, self.current_offset()),
        ));

        Ok(tokens)
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.chars.len()
    }

    fn current_offset(&self) -> usize {
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].0
        } else {
            self.source.len()
        }
    }

    fn peek_char(&self) -> char {
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].1
        } else {
            '\0'
        }
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        if self.cursor + n < self.chars.len() {
            Some(self.chars[self.cursor + n].1)
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let (_, ch) = self.chars[self.cursor];
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_eof() {
            let ch = self.peek_char();
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // Skip '//'
        self.advance();
        self.advance();
        while !self.is_eof() && self.peek_char() != '\n' {
            self.advance();
        }
    }

    fn read_line_doc_comment(&mut self) -> String {
        // Skip '///'
        self.advance();
        self.advance();
        self.advance();
        let mut comment = String::new();
        while !self.is_eof() && self.peek_char() != '\n' {
            if let Some(ch) = self.advance() {
                comment.push(ch);
            }
        }
        comment.trim().to_string()
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        let start_line = self.line;
        let start_col = self.col;
        // Skip '/*'
        self.advance();
        self.advance();
        let mut depth = 1;

        while !self.is_eof() && depth > 0 {
            if self.peek_char() == '/' && self.peek_ahead(1) == Some('*') {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek_char() == '*' && self.peek_ahead(1) == Some('/') {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                self.advance();
            }
        }

        if depth > 0 {
            return Err(SynapseError::LexerError {
                line: start_line,
                col: start_col,
                message: "Unterminated block comment".to_string(),
            });
        }

        Ok(())
    }

    fn read_string(&mut self, span: Span) -> Result<Token> {
        // Skip opening quote
        self.advance();
        let mut content = String::new();

        while !self.is_eof() {
            let ch = self.peek_char();
            if ch == '"' {
                self.advance(); // consume closing quote
                return Ok(Token::new(TokenKind::StringLiteral(content), span));
            } else if ch == '\\' {
                self.advance();
                match self.peek_char() {
                    'n' => { self.advance(); content.push('\n'); }
                    'r' => { self.advance(); content.push('\r'); }
                    't' => { self.advance(); content.push('\t'); }
                    '\\' => { self.advance(); content.push('\\'); }
                    '"' => { self.advance(); content.push('"'); }
                    '0' => { self.advance(); content.push('\0'); }
                    other => {
                        return Err(SynapseError::LexerError {
                            line: self.line,
                            col: self.col,
                            message: format!("Invalid escape sequence '\\{}'", other),
                        });
                    }
                }
            } else if ch == '\n' {
                return Err(SynapseError::LexerError {
                    line: span.line,
                    col: span.col,
                    message: "Unterminated string literal (newline in string)".to_string(),
                });
            } else {
                content.push(ch);
                self.advance();
            }
        }

        Err(SynapseError::LexerError {
            line: span.line,
            col: span.col,
            message: "Unclosed string literal".to_string(),
        })
    }

    fn read_number(&mut self, span: Span) -> Result<Token> {
        let mut s = String::new();
        let is_negative = self.peek_char() == '-';
        if is_negative {
            s.push(self.advance().unwrap());
        }

        // Check for hex
        if self.peek_char() == '0' && (self.peek_ahead(1) == Some('x') || self.peek_ahead(1) == Some('X')) {
            s.push(self.advance().unwrap()); // '0'
            s.push(self.advance().unwrap()); // 'x'
            while !self.is_eof() && self.peek_char().is_ascii_hexdigit() {
                s.push(self.advance().unwrap());
            }
            let hex_str = &s[2..];
            let val = i64::from_str_radix(hex_str, 16).map_err(|e| SynapseError::LexerError {
                line: span.line,
                col: span.col,
                message: format!("Invalid hex literal '{}': {}", s, e),
            })?;
            return Ok(Token::new(TokenKind::IntLiteral(val), span));
        }

        let mut has_dot = false;
        while !self.is_eof() {
            let ch = self.peek_char();
            if ch.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else if ch == '.' && !has_dot && self.peek_ahead(1).map_or(false, |c| c.is_ascii_digit()) {
                has_dot = true;
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if has_dot {
            let val: f64 = s.parse().map_err(|e| SynapseError::LexerError {
                line: span.line,
                col: span.col,
                message: format!("Invalid float literal '{}': {}", s, e),
            })?;
            Ok(Token::new(TokenKind::FloatLiteral(val), span))
        } else {
            let val: i64 = s.parse().map_err(|e| SynapseError::LexerError {
                line: span.line,
                col: span.col,
                message: format!("Invalid integer literal '{}': {}", s, e),
            })?;
            Ok(Token::new(TokenKind::IntLiteral(val), span))
        }
    }

    fn read_ident_or_keyword(&mut self, span: Span) -> Token {
        let mut s = String::new();
        while !self.is_eof() {
            let ch = self.peek_char();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let kind = match s.as_str() {
            "namespace" => TokenKind::Namespace,
            "import" => TokenKind::Import,
            "struct" => TokenKind::Struct,
            "message" => TokenKind::Message,
            "enum" => TokenKind::Enum,
            "service" => TokenKind::Service,
            "rpc" => TokenKind::Rpc,
            "stream" => TokenKind::Stream,
            "pubsub" => TokenKind::PubSub,
            "subscribe" => TokenKind::Subscribe,
            "deprecated" => TokenKind::Deprecated,
            "required" => TokenKind::Required,
            "optional" => TokenKind::Optional,

            "bool" => TokenKind::TypeBool,
            "u8" => TokenKind::TypeU8,
            "u16" => TokenKind::TypeU16,
            "u32" => TokenKind::TypeU32,
            "u64" => TokenKind::TypeU64,
            "i8" => TokenKind::TypeI8,
            "i16" => TokenKind::TypeI16,
            "i32" => TokenKind::TypeI32,
            "i64" => TokenKind::TypeI64,
            "f32" => TokenKind::TypeF32,
            "f64" => TokenKind::TypeF64,
            "string" => TokenKind::TypeString,
            "bytes" => TokenKind::TypeBytes,
            "uuid" => TokenKind::TypeUuid,
            "timestamp" => TokenKind::TypeTimestamp,

            "list" => TokenKind::TypeList,
            "map" => TokenKind::TypeMap,
            "option" => TokenKind::TypeOption,
            "result" => TokenKind::TypeResult,

            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),

            _ => TokenKind::Ident(s),
        };

        Token::new(kind, span)
    }
}
