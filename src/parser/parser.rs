use crate::error::{Result, SynapseError};
use crate::parser::ast::*;
use crate::parser::token::{Span, Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    last_doc_comment: Option<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            last_doc_comment: None,
        }
    }

    pub fn parse_schema(&mut self) -> Result<Schema> {
        let mut schema = Schema::new();

        while !self.is_eof() {
            // Check for doc comments
            if let TokenKind::DocComment(doc) = self.peek_kind() {
                self.advance();
                self.last_doc_comment = Some(doc);
                continue;
            }

            match self.peek_kind() {
                TokenKind::Namespace => {
                    schema.namespace = Some(self.parse_namespace()?);
                }
                TokenKind::Import => {
                    schema.imports.push(self.parse_import()?);
                }
                TokenKind::Struct | TokenKind::Message => {
                    schema.structs.push(self.parse_struct()?);
                }
                TokenKind::Enum => {
                    schema.enums.push(self.parse_enum()?);
                }
                TokenKind::Service => {
                    schema.services.push(self.parse_service()?);
                }
                TokenKind::Eof => break,
                other => {
                    let span = self.current_span();
                    return Err(SynapseError::ParseError {
                        line: span.line,
                        col: span.col,
                        message: format!("Unexpected top-level declaration: {}", other),
                    });
                }
            }
        }

        Ok(schema)
    }

    fn parse_namespace(&mut self) -> Result<Namespace> {
        self.expect(&TokenKind::Namespace)?;
        let mut parts = Vec::new();

        let first = self.expect_ident()?;
        parts.push(first);

        while self.match_kind(&TokenKind::Dot) {
            let part = self.expect_ident()?;
            parts.push(part);
        }

        self.expect(&TokenKind::Semicolon)?;
        self.last_doc_comment = None;
        Ok(Namespace { parts })
    }

    fn parse_import(&mut self) -> Result<Import> {
        self.expect(&TokenKind::Import)?;
        let path = match self.advance().kind {
            TokenKind::StringLiteral(s) => s,
            other => {
                let span = self.current_span();
                return Err(SynapseError::ParseError {
                    line: span.line,
                    col: span.col,
                    message: format!("Expected import path string literal, got {}", other),
                });
            }
        };
        self.expect(&TokenKind::Semicolon)?;
        self.last_doc_comment = None;
        Ok(Import { path })
    }

    fn parse_struct(&mut self) -> Result<StructDef> {
        let doc = self.last_doc_comment.take();
        if self.match_kind(&TokenKind::Struct) || self.match_kind(&TokenKind::Message) {
            // consumed
        } else {
            let span = self.current_span();
            return Err(SynapseError::ParseError {
                line: span.line,
                col: span.col,
                message: "Expected 'struct' or 'message'".to_string(),
            });
        }

        let name = self.expect_ident()?;
        let attributes = self.parse_attributes()?;

        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            if let TokenKind::DocComment(comment) = self.peek_kind() {
                self.advance();
                self.last_doc_comment = Some(comment);
                continue;
            }
            fields.push(self.parse_field()?);
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(StructDef {
            name,
            fields,
            attributes,
            doc,
        })
    }

    fn parse_field(&mut self) -> Result<Field> {
        let doc = self.last_doc_comment.take();
        let field_id = match self.advance().kind {
            TokenKind::IntLiteral(n) if n >= 0 => n as u32,
            other => {
                let span = self.current_span();
                return Err(SynapseError::ParseError {
                    line: span.line,
                    col: span.col,
                    message: format!("Expected non-negative integer field ID, got {}", other),
                });
            }
        };

        self.expect(&TokenKind::Colon)?;

        let modifier = if self.match_kind(&TokenKind::Optional) {
            FieldModifier::Optional
        } else if self.match_kind(&TokenKind::Required) {
            FieldModifier::Required
        } else {
            FieldModifier::Required
        };

        let field_type = self.parse_field_type()?;
        let name = self.expect_ident()?;

        let default_value = if self.match_kind(&TokenKind::Equals) {
            Some(self.parse_literal_value()?)
        } else {
            None
        };

        let attributes = self.parse_attributes()?;
        self.expect(&TokenKind::Semicolon)?;

        Ok(Field {
            id: field_id,
            name,
            field_type,
            modifier,
            default_value,
            attributes,
            doc,
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDef> {
        let doc = self.last_doc_comment.take();
        self.expect(&TokenKind::Enum)?;
        let name = self.expect_ident()?;
        let attributes = self.parse_attributes()?;

        self.expect(&TokenKind::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            if let TokenKind::DocComment(comment) = self.peek_kind() {
                self.advance();
                self.last_doc_comment = Some(comment);
                continue;
            }

            let variant_doc = self.last_doc_comment.take();
            let variant_name = self.expect_ident()?;
            self.expect(&TokenKind::Equals)?;

            let val = match self.advance().kind {
                TokenKind::IntLiteral(n) => n as i32,
                other => {
                    let span = self.current_span();
                    return Err(SynapseError::ParseError {
                        line: span.line,
                        col: span.col,
                        message: format!("Expected integer enum value, got {}", other),
                    });
                }
            };

            let variant_attrs = self.parse_attributes()?;
            self.match_kind(&TokenKind::Comma);

            variants.push(EnumVariant {
                name: variant_name,
                value: val,
                doc: variant_doc,
                attributes: variant_attrs,
            });
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(EnumDef {
            name,
            variants,
            attributes,
            doc,
        })
    }

    fn parse_service(&mut self) -> Result<ServiceDef> {
        let doc = self.last_doc_comment.take();
        self.expect(&TokenKind::Service)?;
        let name = self.expect_ident()?;
        let attributes = self.parse_attributes()?;

        self.expect(&TokenKind::LBrace)?;
        let mut methods = Vec::new();
        let mut method_counter = 1u32;

        while !self.check(&TokenKind::RBrace) && !self.is_eof() {
            if let TokenKind::DocComment(comment) = self.peek_kind() {
                self.advance();
                self.last_doc_comment = Some(comment);
                continue;
            }

            let method_doc = self.last_doc_comment.take();
            let is_pubsub = self.match_kind(&TokenKind::Subscribe) || self.match_kind(&TokenKind::PubSub);
            if !is_pubsub {
                self.expect(&TokenKind::Rpc)?;
            }

            let method_name = self.expect_ident()?;
            self.expect(&TokenKind::LParen)?;

            let client_stream = self.match_kind(&TokenKind::Stream);
            let request_type = self.parse_field_type()?;
            self.expect(&TokenKind::RParen)?;

            self.expect(&TokenKind::Arrow)?;

            let server_stream = is_pubsub || self.match_kind(&TokenKind::Stream);
            let response_type = self.parse_field_type()?;

            let streaming_mode = match (client_stream, server_stream) {
                (false, false) => StreamingMode::Unary,
                (false, true) => StreamingMode::ServerStreaming,
                (true, false) => StreamingMode::ClientStreaming,
                (true, true) => StreamingMode::BiDirectional,
            };

            let method_attrs = self.parse_attributes()?;
            self.expect(&TokenKind::Semicolon)?;

            methods.push(MethodDef {
                id: method_counter,
                name: method_name,
                request_type,
                response_type,
                streaming_mode,
                attributes: method_attrs,
                doc: method_doc,
            });

            method_counter += 1;
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(ServiceDef {
            name,
            methods,
            attributes,
            doc,
        })
    }

    fn parse_field_type(&mut self) -> Result<FieldType> {
        match self.peek_kind() {
            TokenKind::TypeBool => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::Bool)) }
            TokenKind::TypeU8 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::U8)) }
            TokenKind::TypeU16 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::U16)) }
            TokenKind::TypeU32 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::U32)) }
            TokenKind::TypeU64 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::U64)) }
            TokenKind::TypeI8 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::I8)) }
            TokenKind::TypeI16 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::I16)) }
            TokenKind::TypeI32 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::I32)) }
            TokenKind::TypeI64 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::I64)) }
            TokenKind::TypeF32 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::F32)) }
            TokenKind::TypeF64 => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::F64)) }
            TokenKind::TypeString => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::String)) }
            TokenKind::TypeBytes => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::Bytes)) }
            TokenKind::TypeUuid => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::Uuid)) }
            TokenKind::TypeTimestamp => { self.advance(); Ok(FieldType::Primitive(PrimitiveType::Timestamp)) }

            TokenKind::TypeList => {
                self.advance();
                self.expect(&TokenKind::LAngle)?;
                let inner = self.parse_field_type()?;
                self.expect(&TokenKind::RAngle)?;
                Ok(FieldType::List(Box::new(inner)))
            }
            TokenKind::TypeOption => {
                self.advance();
                self.expect(&TokenKind::LAngle)?;
                let inner = self.parse_field_type()?;
                self.expect(&TokenKind::RAngle)?;
                Ok(FieldType::Option(Box::new(inner)))
            }
            TokenKind::TypeMap => {
                self.advance();
                self.expect(&TokenKind::LAngle)?;
                let key_type = self.parse_field_type()?;
                self.expect(&TokenKind::Comma)?;
                let val_type = self.parse_field_type()?;
                self.expect(&TokenKind::RAngle)?;
                Ok(FieldType::Map(Box::new(key_type), Box::new(val_type)))
            }
            TokenKind::TypeResult => {
                self.advance();
                self.expect(&TokenKind::LAngle)?;
                let ok_type = self.parse_field_type()?;
                self.expect(&TokenKind::Comma)?;
                let err_type = self.parse_field_type()?;
                self.expect(&TokenKind::RAngle)?;
                Ok(FieldType::Result {
                    ok: Box::new(ok_type),
                    err: Box::new(err_type),
                })
            }
            TokenKind::Ident(ident) => {
                let name = ident.clone();
                self.advance();
                Ok(FieldType::Custom(name))
            }
            other => {
                let span = self.current_span();
                Err(SynapseError::ParseError {
                    line: span.line,
                    col: span.col,
                    message: format!("Expected type name, found {}", other),
                })
            }
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>> {
        let mut attrs = Vec::new();
        while self.match_kind(&TokenKind::At) {
            let name = self.expect_ident()?;
            let mut args = Vec::new();
            if self.match_kind(&TokenKind::LParen) {
                while !self.check(&TokenKind::RParen) && !self.is_eof() {
                    args.push(self.parse_literal_value()?);
                    self.match_kind(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RParen)?;
            }
            attrs.push(Attribute { name, args });
        }
        Ok(attrs)
    }

    fn parse_literal_value(&mut self) -> Result<LiteralValue> {
        let token = self.advance();
        match token.kind {
            TokenKind::StringLiteral(s) => Ok(LiteralValue::String(s)),
            TokenKind::IntLiteral(n) => Ok(LiteralValue::Int(n)),
            TokenKind::FloatLiteral(f) => Ok(LiteralValue::Float(f)),
            TokenKind::BoolLiteral(b) => Ok(LiteralValue::Bool(b)),
            other => {
                let span = self.current_span();
                Err(SynapseError::ParseError {
                    line: span.line,
                    col: span.col,
                    message: format!("Expected literal value, found {}", other),
                })
            }
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        let token = self.advance();
        match token.kind {
            TokenKind::Ident(s) => Ok(s),
            other => Err(SynapseError::ParseError {
                line: token.span.line,
                col: token.span.col,
                message: format!("Expected identifier, found {}", other),
            }),
        }
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<()> {
        let token = self.advance();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(expected) {
            Ok(())
        } else {
            Err(SynapseError::ParseError {
                line: token.span.line,
                col: token.span.col,
                message: format!("Expected {}, found {}", expected, token.kind),
            })
        }
    }

    fn match_kind(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(kind)
    }

    fn peek_kind(&self) -> TokenKind {
        if self.cursor < self.tokens.len() {
            self.tokens[self.cursor].kind.clone()
        } else {
            TokenKind::Eof
        }
    }

    fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            Token::new(TokenKind::Eof, Span::new(0, 0, 0))
        }
    }

    fn is_eof(&self) -> bool {
        self.cursor >= self.tokens.len() || self.peek_kind() == TokenKind::Eof
    }

    fn current_span(&self) -> Span {
        if self.cursor < self.tokens.len() {
            self.tokens[self.cursor].span.clone()
        } else {
            Span::new(0, 0, 0)
        }
    }
}
