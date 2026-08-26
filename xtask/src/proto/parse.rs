//! The `.proto` parser.
//!
//! Produces just enough of a schema to generate wire code: messages, enums,
//! fields and services. Everything that only describes the schema — custom
//! options, `extend` blocks declaring them, `reserved` ranges — is recognised and
//! discarded, because none of it appears on the wire.
//!
//! Anything *not* recognised is an error rather than a skip. A generator that
//! silently ignored a construct would emit code that disagrees with the schema,
//! and the disagreement would surface as Steam rejecting a message for reasons
//! no log line explains.

use super::lex::{LexError, Spanned, Token, lex};
use std::fmt;

/// A parsed `.proto` file.
#[derive(Debug, Clone, Default)]
pub struct ProtoFile {
    /// The file's name, for diagnostics and for grouping generated output.
    pub name: String,
    /// The `package` declaration, if any. Most of Valve's files have none.
    pub package: Option<String>,
    /// Top-level messages.
    pub messages: Vec<Message>,
    /// Top-level enums.
    pub enums: Vec<Enum>,
    /// Service definitions, which name the unified-message RPCs.
    pub services: Vec<Service>,
}

/// A message definition, possibly with nested types.
#[derive(Debug, Clone, Default)]
pub struct Message {
    /// The message's own name, unqualified.
    pub name: String,
    /// Its fields, including those flattened out of `oneof` blocks.
    pub fields: Vec<Field>,
    /// `oneof` groupings, by name, listing the field numbers they contain.
    pub oneofs: Vec<Oneof>,
    /// Nested messages.
    pub messages: Vec<Message>,
    /// Nested enums.
    pub enums: Vec<Enum>,
}

/// A `oneof` block.
///
/// The fields inside it are stored on the message like any others; this records
/// only the grouping. Protobuf's own wire format does the same — a `oneof` is a
/// set of ordinary fields plus a rule that at most one is set — so generated
/// code writes whichever is populated and reading the second one clears the
/// first.
#[derive(Debug, Clone, Default)]
pub struct Oneof {
    /// The block's name.
    ///
    /// Unused by the generator today: a `oneof` is ordinary fields plus a rule
    /// that at most one is set, and the wire format carries no trace of the
    /// grouping. Kept because it is what a future accessor that enforces the
    /// rule would be named after.
    #[allow(
        dead_code,
        reason = "recorded from the schema; the generator does not need it yet"
    )]
    pub name: String,
    /// The field numbers it contains.
    pub field_numbers: Vec<u32>,
}

/// How many times a field may appear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    /// Zero or one.
    Optional,
    /// Exactly one. Valve still uses this in a few older messages.
    Required,
    /// Any number.
    Repeated,
}

/// A field's type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// A built-in scalar.
    Scalar(Scalar),
    /// A reference to a message or enum, resolved later.
    Named(String),
}

/// The protobuf scalar types, spelled as the specification spells them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    missing_docs,
    reason = "the protobuf specification names these, not us"
)]
pub enum Scalar {
    Double,
    Float,
    Int32,
    Int64,
    Uint32,
    Uint64,
    Sint32,
    Sint64,
    Fixed32,
    Fixed64,
    Sfixed32,
    Sfixed64,
    Bool,
    String,
    Bytes,
}

impl Scalar {
    /// Matches a type keyword.
    fn from_keyword(word: &str) -> Option<Self> {
        Some(match word {
            "double" => Self::Double,
            "float" => Self::Float,
            "int32" => Self::Int32,
            "int64" => Self::Int64,
            "uint32" => Self::Uint32,
            "uint64" => Self::Uint64,
            "sint32" => Self::Sint32,
            "sint64" => Self::Sint64,
            "fixed32" => Self::Fixed32,
            "fixed64" => Self::Fixed64,
            "sfixed32" => Self::Sfixed32,
            "sfixed64" => Self::Sfixed64,
            "bool" => Self::Bool,
            "string" => Self::String,
            "bytes" => Self::Bytes,
            _ => return None,
        })
    }
}

/// A single field.
#[derive(Debug, Clone)]
pub struct Field {
    /// Its name as written.
    pub name: String,
    /// Its cardinality.
    pub label: Label,
    /// Its type.
    pub ty: FieldType,
    /// Its wire number.
    pub number: u32,
    /// The `[default = ...]` value, verbatim.
    pub default: Option<String>,
    /// Whether `[packed = true]` was given.
    ///
    /// proto2 does not pack by default, so this changes what the encoder emits.
    /// The decoder accepts both forms regardless, as protobuf requires.
    pub packed: bool,
}

/// An enum definition.
#[derive(Debug, Clone, Default)]
pub struct Enum {
    /// The enum's name.
    pub name: String,
    /// Its values, in declaration order. Duplicates are possible: Valve uses
    /// aliases.
    pub values: Vec<(String, i64)>,
}

/// A service definition.
#[derive(Debug, Clone, Default)]
pub struct Service {
    /// The service's name, which is the first half of a unified-message target
    /// such as `Authentication.BeginAuthSessionViaCredentials`.
    pub name: String,
    /// Its methods.
    pub methods: Vec<Method>,
}

/// An RPC method.
#[derive(Debug, Clone)]
pub struct Method {
    /// The method's name.
    pub name: String,
    /// The request message type.
    pub input: String,
    /// The response message type.
    pub output: String,
}

/// A parse failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What went wrong.
    pub message: String,
    /// 1-based line number, where known.
    pub line: u32,
    /// The file it happened in.
    pub file: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

impl From<(LexError, &str)> for ParseError {
    fn from((error, file): (LexError, &str)) -> Self {
        Self {
            message: error.message,
            line: error.line,
            file: file.to_owned(),
        }
    }
}

/// Parses one `.proto` file.
pub fn parse(name: &str, source: &str) -> Result<ProtoFile, ParseError> {
    let tokens = lex(source).map_err(|e| ParseError::from((e, name)))?;
    Parser {
        tokens,
        pos: 0,
        file: name.to_owned(),
    }
    .parse_file(name)
}

struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
    file: String,
}

impl Parser {
    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            line: self
                .tokens
                .get(self.pos.min(self.tokens.len().saturating_sub(1)))
                .map_or(0, |s| s.line),
            file: self.file.clone(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|s| &s.token)
    }

    fn next(&mut self) -> Result<Token, ParseError> {
        let token = self
            .tokens
            .get(self.pos)
            .map(|s| s.token.clone())
            .ok_or_else(|| self.error("unexpected end of file"))?;
        self.pos += 1;
        Ok(token)
    }

    fn expect_punct(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next()? {
            Token::Punct(c) if c == expected => Ok(()),
            other => Err(self.error(format!("expected '{expected}', found {other}"))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.next()? {
            Token::Ident(name) => Ok(name),
            other => Err(self.error(format!("expected an identifier, found {other}"))),
        }
    }

    /// Consumes the identifier `word` if it is next.
    fn eat_ident(&mut self, word: &str) -> bool {
        if matches!(self.peek(), Some(Token::Ident(i)) if i == word) {
            self.pos += 1;
            return true;
        }
        false
    }

    /// Consumes the punctuation `c` if it is next.
    fn eat_punct(&mut self, c: char) -> bool {
        if matches!(self.peek(), Some(Token::Punct(p)) if *p == c) {
            self.pos += 1;
            return true;
        }
        false
    }

    /// Skips forward past the next `;`, used to discard statements whose
    /// contents do not affect the wire format.
    fn skip_to_semicolon(&mut self) -> Result<(), ParseError> {
        loop {
            match self.next()? {
                Token::Punct(';') => return Ok(()),
                Token::Punct('{') => self.skip_braced_block()?,
                _ => {}
            }
        }
    }

    /// Skips a `{ ... }` block whose opening brace has already been consumed.
    fn skip_braced_block(&mut self) -> Result<(), ParseError> {
        let mut depth = 1_u32;
        while depth > 0 {
            match self.next()? {
                Token::Punct('{') => depth += 1,
                Token::Punct('}') => depth -= 1,
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_file(mut self, name: &str) -> Result<ProtoFile, ParseError> {
        let mut file = ProtoFile {
            name: name.to_owned(),
            ..ProtoFile::default()
        };

        while self.peek().is_some() {
            // Stray semicolons are legal.
            if self.eat_punct(';') {
                continue;
            }

            let keyword = match self.peek() {
                Some(Token::Ident(word)) => word.clone(),
                Some(other) => {
                    let other = other.clone();
                    return Err(self.error(format!("expected a declaration, found {other}")));
                }
                None => break,
            };

            match keyword.as_str() {
                "syntax" | "import" | "option" => {
                    self.pos += 1;
                    self.skip_to_semicolon()?;
                }
                "package" => {
                    self.pos += 1;
                    file.package = Some(self.expect_ident()?);
                    self.expect_punct(';')?;
                }
                "extend" => {
                    // Only ever used to declare custom options, which describe
                    // the schema and never appear on the wire.
                    self.pos += 1;
                    let _target = self.expect_ident()?;
                    self.expect_punct('{')?;
                    self.skip_braced_block()?;
                }
                "message" => {
                    self.pos += 1;
                    file.messages.push(self.parse_message()?);
                }
                "enum" => {
                    self.pos += 1;
                    file.enums.push(self.parse_enum()?);
                }
                "service" => {
                    self.pos += 1;
                    file.services.push(self.parse_service()?);
                }
                other => {
                    return Err(self.error(format!("unsupported declaration `{other}`")));
                }
            }
        }

        Ok(file)
    }

    fn parse_message(&mut self) -> Result<Message, ParseError> {
        let mut message = Message {
            name: self.expect_ident()?,
            ..Message::default()
        };
        self.expect_punct('{')?;

        loop {
            if self.eat_punct('}') {
                return Ok(message);
            }
            if self.eat_punct(';') {
                continue;
            }

            let keyword = match self.peek() {
                Some(Token::Ident(word)) => word.clone(),
                Some(other) => {
                    let other = other.clone();
                    return Err(self.error(format!("expected a field, found {other}")));
                }
                None => return Err(self.error("unterminated message")),
            };

            match keyword.as_str() {
                "message" => {
                    self.pos += 1;
                    message.messages.push(self.parse_message()?);
                }
                "enum" => {
                    self.pos += 1;
                    message.enums.push(self.parse_enum()?);
                }
                "oneof" => {
                    self.pos += 1;
                    let oneof = self.parse_oneof(&mut message)?;
                    message.oneofs.push(oneof);
                }
                "option" | "reserved" | "extensions" => {
                    self.pos += 1;
                    self.skip_to_semicolon()?;
                }
                "extend" => {
                    self.pos += 1;
                    let _target = self.expect_ident()?;
                    self.expect_punct('{')?;
                    self.skip_braced_block()?;
                }
                _ => {
                    let field = self.parse_field(None)?;
                    message.fields.push(field);
                }
            }
        }
    }

    /// Parses a `oneof` block, appending its fields to the enclosing message.
    fn parse_oneof(&mut self, message: &mut Message) -> Result<Oneof, ParseError> {
        let mut oneof = Oneof {
            name: self.expect_ident()?,
            ..Oneof::default()
        };
        self.expect_punct('{')?;

        loop {
            if self.eat_punct('}') {
                return Ok(oneof);
            }
            if self.eat_punct(';') {
                continue;
            }
            if self.eat_ident("option") {
                self.skip_to_semicolon()?;
                continue;
            }
            // Fields inside a oneof carry no label.
            let field = self.parse_field(Some(Label::Optional))?;
            oneof.field_numbers.push(field.number);
            message.fields.push(field);
        }
    }

    /// Parses one field. `forced_label` is set inside a `oneof`, where the
    /// label is omitted.
    fn parse_field(&mut self, forced_label: Option<Label>) -> Result<Field, ParseError> {
        let label = match forced_label {
            Some(label) => label,
            None => {
                let word = self.expect_ident()?;
                match word.as_str() {
                    "optional" => Label::Optional,
                    "required" => Label::Required,
                    "repeated" => Label::Repeated,
                    other => {
                        return Err(self.error(format!(
                            "expected a field label, found `{other}` (groups and maps are not \
                             supported, and Valve's schema contains neither)"
                        )));
                    }
                }
            }
        };

        let type_name = self.expect_ident()?;
        let ty = match Scalar::from_keyword(&type_name) {
            Some(scalar) => FieldType::Scalar(scalar),
            None => FieldType::Named(type_name),
        };

        let name = self.expect_ident()?;
        self.expect_punct('=')?;

        let number = match self.next()? {
            Token::Int(n) if n > 0 => u32::try_from(n)
                .map_err(|_| self.error(format!("field number {n} is out of range")))?,
            other => return Err(self.error(format!("expected a field number, found {other}"))),
        };

        let mut default = None;
        let mut packed = false;

        if self.eat_punct('[') {
            loop {
                // An option name is either a bare identifier or a parenthesised
                // custom one such as `(description)`.
                let option_name = if self.eat_punct('(') {
                    let name = self.expect_ident()?;
                    self.expect_punct(')')?;
                    name
                } else {
                    self.expect_ident()?
                };

                self.expect_punct('=')?;
                let value = self.next()?;

                match option_name.as_str() {
                    "default" => default = Some(value.to_string()),
                    "packed" => packed = matches!(&value, Token::Ident(v) if v == "true"),
                    _ => {}
                }

                if self.eat_punct(',') {
                    continue;
                }
                self.expect_punct(']')?;
                break;
            }
        }

        self.expect_punct(';')?;

        Ok(Field {
            name,
            label,
            ty,
            number,
            default,
            packed,
        })
    }

    fn parse_enum(&mut self) -> Result<Enum, ParseError> {
        let mut definition = Enum {
            name: self.expect_ident()?,
            ..Enum::default()
        };
        self.expect_punct('{')?;

        loop {
            if self.eat_punct('}') {
                return Ok(definition);
            }
            if self.eat_punct(';') {
                continue;
            }
            if self.eat_ident("option") || self.eat_ident("reserved") {
                self.skip_to_semicolon()?;
                continue;
            }

            let name = self.expect_ident()?;
            self.expect_punct('=')?;
            let value = match self.next()? {
                // Enum values are int32 on the wire, so anything wider is a
                // schema we do not understand rather than a value to truncate.
                Token::Int(n) => i64::try_from(n)
                    .map_err(|_| self.error(format!("enum value {n} is out of range")))?,
                other => return Err(self.error(format!("expected an enum value, found {other}"))),
            };

            // Enum values may carry options too, such as `[(enum_description)]`.
            if self.eat_punct('[') {
                loop {
                    match self.next()? {
                        Token::Punct(']') => break,
                        Token::Punct('[') => {
                            return Err(self.error("nested '[' in an enum value option"));
                        }
                        _ => {}
                    }
                }
            }
            self.expect_punct(';')?;
            definition.values.push((name, value));
        }
    }

    fn parse_service(&mut self) -> Result<Service, ParseError> {
        let mut service = Service {
            name: self.expect_ident()?,
            ..Service::default()
        };
        self.expect_punct('{')?;

        loop {
            if self.eat_punct('}') {
                return Ok(service);
            }
            if self.eat_punct(';') {
                continue;
            }
            if self.eat_ident("option") {
                self.skip_to_semicolon()?;
                continue;
            }

            if !self.eat_ident("rpc") {
                let found = self.next()?;
                return Err(self.error(format!("expected `rpc`, found {found}")));
            }

            let name = self.expect_ident()?;
            self.expect_punct('(')?;
            let input = self.expect_ident()?;
            self.expect_punct(')')?;

            if !self.eat_ident("returns") {
                return Err(self.error("expected `returns`"));
            }
            self.expect_punct('(')?;
            let output = self.expect_ident()?;
            self.expect_punct(')')?;

            // A method body holds only options.
            if self.eat_punct('{') {
                self.skip_braced_block()?;
            } else {
                self.expect_punct(';')?;
            }

            service.methods.push(Method {
                name,
                input,
                output,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_message_with_nested_types() {
        let source = r#"
            message ContentManifestPayload {
                message FileMapping {
                    message ChunkData {
                        optional bytes sha = 1;
                        optional fixed32 crc = 2;
                    }
                    optional string filename = 1;
                    repeated .ContentManifestPayload.FileMapping.ChunkData chunks = 6;
                }
                repeated .ContentManifestPayload.FileMapping mappings = 1;
            }
        "#;
        let file = parse("test.proto", source).expect("must parse");
        let payload = file.messages.first().expect("one message");
        assert_eq!(payload.name, "ContentManifestPayload");

        let mapping = payload.messages.first().expect("a nested message");
        assert_eq!(mapping.name, "FileMapping");
        assert_eq!(
            mapping.messages.first().map(|m| m.name.as_str()),
            Some("ChunkData")
        );

        let chunks = mapping
            .fields
            .iter()
            .find(|f| f.name == "chunks")
            .expect("chunks field");
        assert_eq!(chunks.label, Label::Repeated);
        assert_eq!(
            chunks.ty,
            FieldType::Named(".ContentManifestPayload.FileMapping.ChunkData".into())
        );
        assert_eq!(chunks.number, 6);
    }

    #[test]
    fn reads_defaults_and_packed() {
        let source = r#"
            message M {
                optional uint32 a = 1 [default = 5];
                optional bool b = 2 [default = true];
                repeated uint32 c = 3 [packed = true];
                optional string d = 4 [(description) = "ignored", default = "hi"];
            }
        "#;
        let file = parse("test.proto", source).expect("must parse");
        let m = file.messages.first().expect("one message");
        assert_eq!(
            m.fields.first().and_then(|f| f.default.clone()),
            Some("5".into())
        );
        assert_eq!(
            m.fields.get(1).and_then(|f| f.default.clone()),
            Some("true".into())
        );
        assert!(m.fields.get(2).is_some_and(|f| f.packed));
        assert_eq!(
            m.fields.get(3).and_then(|f| f.default.clone()),
            Some("\"hi\"".into())
        );
    }

    #[test]
    fn oneof_fields_land_on_the_message_and_are_recorded() {
        let source = r#"
            message M {
                oneof body {
                    uint32 a = 1;
                    string b = 2;
                }
            }
        "#;
        let file = parse("test.proto", source).expect("must parse");
        let m = file.messages.first().expect("one message");
        assert_eq!(m.fields.len(), 2);
        assert_eq!(
            m.oneofs.first().map(|o| o.field_numbers.clone()),
            Some(vec![1, 2])
        );
    }

    #[test]
    fn services_yield_their_rpc_names() {
        let source = r#"
            service Authentication {
                rpc BeginAuthSessionViaCredentials (.CAuthentication_Request) returns (.CAuthentication_Response);
                rpc PollAuthSessionStatus (.CPoll_Request) returns (.CPoll_Response) {
                    option (method_description) = "poll";
                }
            }
        "#;
        let file = parse("test.proto", source).expect("must parse");
        let service = file.services.first().expect("one service");
        assert_eq!(service.name, "Authentication");
        assert_eq!(service.methods.len(), 2);
        assert_eq!(
            service.methods.first().map(|m| m.name.as_str()),
            Some("BeginAuthSessionViaCredentials")
        );
    }

    #[test]
    fn schema_only_constructs_are_discarded() {
        // extend blocks declare custom options; reserved and extensions describe
        // field-number policy. None of it reaches the wire.
        let source = r#"
            import "google/protobuf/descriptor.proto";
            option optimize_for = SPEED;
            extend google.protobuf.FieldOptions {
                optional string description = 50000;
            }
            message M {
                reserved 2, 15, 9 to 11;
                extensions 100 to 199;
                option (msg_option) = true;
                optional uint32 a = 1;
            }
        "#;
        let file = parse("test.proto", source).expect("must parse");
        let m = file.messages.first().expect("one message");
        assert_eq!(m.fields.len(), 1);
    }

    #[test]
    fn an_unsupported_construct_is_an_error_not_a_skip() {
        // If Valve ever adds a map field, the generator must stop rather than
        // emit code that disagrees with the schema.
        let err = parse("test.proto", "message M { map<string, uint32> m = 1; }")
            .expect_err("must refuse");
        assert!(
            err.message.contains("map"),
            "unhelpful message: {}",
            err.message
        );
    }

    #[test]
    fn enum_values_including_negative_ones_are_read() {
        let source = r#"
            enum EResult {
                k_EResultInvalid = 0;
                k_EResultOK = 1;
                k_EResultNegative = -1;
            }
        "#;
        let file = parse("test.proto", source).expect("must parse");
        let e = file.enums.first().expect("one enum");
        assert_eq!(e.values.len(), 3);
        assert_eq!(e.values.get(2).map(|(_, v)| *v), Some(-1));
    }
}
