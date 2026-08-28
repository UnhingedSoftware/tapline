//! The KeyValues text parser: no `#include`, no `#base`, no macros.

use crate::{MAX_DEPTH, Object, Value};
use std::fmt;

/// What went wrong reading a KeyValues file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdfError {
    /// The file ended inside a string, a block, or before a value.
    UnexpectedEnd,
    /// A closing brace with no matching opening brace.
    UnmatchedBrace {
        /// Byte offset of the offending brace.
        offset: usize,
    },
    /// Nesting went past [`MAX_DEPTH`].
    DepthLimitExceeded {
        /// Byte offset where the limit was hit.
        offset: usize,
    },
    /// A `{` where a key was expected.
    UnexpectedBrace {
        /// Byte offset of the offending brace.
        offset: usize,
    },
    /// A directive this parser does not implement, such as `#include`.
    UnsupportedDirective {
        /// The directive as written.
        directive: String,
        /// Byte offset where it appeared.
        offset: usize,
    },
}

impl fmt::Display for VdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd => f.write_str("file ended mid-value"),
            Self::UnmatchedBrace { offset } => write!(f, "unmatched '}}' at byte {offset}"),
            Self::DepthLimitExceeded { offset } => {
                write!(f, "nesting deeper than {MAX_DEPTH} at byte {offset}")
            }
            Self::UnexpectedBrace { offset } => write!(f, "unexpected '{{' at byte {offset}"),
            Self::UnsupportedDirective { directive, offset } => {
                write!(f, "unsupported directive {directive} at byte {offset}")
            }
        }
    }
}

impl std::error::Error for VdfError {}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Str(String),
    Open,
    Close,
}

struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    const fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    /// Skips whitespace, `//` comments, and `[$CONDITION]` tags (dropped, not evaluated).
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(b) if b.is_ascii_whitespace() => self.pos += 1,
                Some(b'/') if self.input.get(self.pos + 1) == Some(&b'/') => {
                    while let Some(b) = self.peek() {
                        self.pos += 1;
                        if b == b'\n' {
                            break;
                        }
                    }
                }
                Some(b'[') => {
                    while let Some(b) = self.peek() {
                        self.pos += 1;
                        if b == b']' {
                            break;
                        }
                    }
                }
                _ => return,
            }
        }
    }

    /// Decoded as UTF-8 at the end: per-byte conversion would split multi-byte characters.
    fn read_quoted(&mut self) -> Result<String, VdfError> {
        self.pos += 1; // opening quote
        let mut out: Vec<u8> = Vec::new();
        loop {
            let byte = self.peek().ok_or(VdfError::UnexpectedEnd)?;
            self.pos += 1;
            match byte {
                b'"' => return Ok(String::from_utf8_lossy(&out).into_owned()),
                b'\\' => {
                    let escaped = self.peek().ok_or(VdfError::UnexpectedEnd)?;
                    self.pos += 1;
                    out.push(match escaped {
                        b'n' => b'\n',
                        b't' => b'\t',
                        b'r' => b'\r',
                        // Anything else stands for itself, matching Valve's reader.
                        other => other,
                    });
                }
                other => out.push(other),
            }
        }
    }

    fn read_bare(&mut self) -> String {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() || b == b'{' || b == b'}' || b == b'"' {
                break;
            }
            self.pos += 1;
        }
        // In bounds by construction; the fallback keeps the no-panic rule.
        self.input
            .get(start..self.pos)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_default()
    }

    fn next_token(&mut self) -> Result<Option<(usize, Token)>, VdfError> {
        self.skip_trivia();
        let offset = self.pos;
        match self.peek() {
            None => Ok(None),
            Some(b'{') => {
                self.pos += 1;
                Ok(Some((offset, Token::Open)))
            }
            Some(b'}') => {
                self.pos += 1;
                Ok(Some((offset, Token::Close)))
            }
            Some(b'"') => Ok(Some((offset, Token::Str(self.read_quoted()?)))),
            Some(b'#') => {
                let directive = self.read_bare();
                Err(VdfError::UnsupportedDirective { directive, offset })
            }
            Some(_) => Ok(Some((offset, Token::Str(self.read_bare())))),
        }
    }
}

/// Parses a KeyValues document into its top-level pairs.
pub fn parse(input: &str) -> Result<Object, VdfError> {
    let mut lexer = Lexer::new(input);
    let object = parse_object(&mut lexer, 0, true)?;
    Ok(object)
}

/// Parses pairs until the matching `}`, or end of input at the top level.
fn parse_object(lexer: &mut Lexer<'_>, depth: u32, top_level: bool) -> Result<Object, VdfError> {
    let mut object = Object::new();

    loop {
        let Some((offset, token)) = lexer.next_token()? else {
            if top_level {
                return Ok(object);
            }
            return Err(VdfError::UnexpectedEnd);
        };

        let key = match token {
            Token::Close if !top_level => return Ok(object),
            Token::Close => return Err(VdfError::UnmatchedBrace { offset }),
            Token::Open => return Err(VdfError::UnexpectedBrace { offset }),
            Token::Str(key) => key,
        };

        let Some((value_offset, value_token)) = lexer.next_token()? else {
            return Err(VdfError::UnexpectedEnd);
        };

        let value = match value_token {
            Token::Str(s) => Value::String(s),
            Token::Open => {
                if depth >= MAX_DEPTH {
                    return Err(VdfError::DepthLimitExceeded {
                        offset: value_offset,
                    });
                }
                Value::Object(parse_object(lexer, depth + 1, false)?)
            }
            Token::Close => {
                return Err(VdfError::UnmatchedBrace {
                    offset: value_offset,
                });
            }
        };

        object.push(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_flat_block() {
        let obj = parse("\"AppState\"\n{\n\t\"appid\"\t\t\"232250\"\n}\n").expect("must parse");
        let state = obj
            .get_object("AppState")
            .expect("AppState must be an object");
        assert_eq!(state.get_str("appid"), Some("232250"));
        assert_eq!(state.get_u64("appid"), Some(232_250));
    }

    #[test]
    fn key_lookup_ignores_case() {
        let obj = parse("\"a\" { \"AppID\" \"7\" }").expect("must parse");
        let a = obj.get_object("A").expect("must find a");
        assert_eq!(a.get_str("appid"), Some("7"));
    }

    #[test]
    fn duplicate_keys_are_all_kept() {
        let obj = parse("\"k\" \"1\" \"k\" \"2\"").expect("must parse");
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get_str("k"), Some("1"));
        let all: Vec<_> = obj.iter().filter_map(|(_, v)| v.as_str()).collect();
        assert_eq!(all, vec!["1", "2"]);
    }

    #[test]
    fn comments_and_conditionals_are_skipped() {
        let obj =
            parse("// leading comment\n\"k\" \"v\" [$WIN32]\n// trailing\n").expect("must parse");
        assert_eq!(obj.get_str("k"), Some("v"));
    }

    #[test]
    fn escapes_are_resolved() {
        let obj = parse(r#""k" "line\nbreak\tand \"quotes\"""#).expect("must parse");
        assert_eq!(obj.get_str("k"), Some("line\nbreak\tand \"quotes\""));
    }

    #[test]
    fn bare_tokens_are_accepted() {
        // Some hand-edited server configs in the wild use bare tokens.
        let obj = parse("key value").expect("must parse");
        assert_eq!(obj.get_str("key"), Some("value"));
    }

    #[test]
    fn missing_and_non_numeric_fields_report_none_not_zero() {
        let obj = parse("\"size\" \"not a number\"").expect("must parse");
        assert_eq!(obj.get_u64("size"), None);
        assert_eq!(obj.get_u64("absent"), None);
    }

    #[test]
    fn truncated_input_is_an_error_not_a_partial_parse() {
        assert_eq!(parse("\"k\" {"), Err(VdfError::UnexpectedEnd));
        assert_eq!(parse("\"k\""), Err(VdfError::UnexpectedEnd));
        assert_eq!(parse("\"unterminated"), Err(VdfError::UnexpectedEnd));
    }

    #[test]
    fn unmatched_closing_brace_is_rejected() {
        assert!(matches!(
            parse("\"k\" \"v\" }"),
            Err(VdfError::UnmatchedBrace { .. })
        ));
    }

    #[test]
    fn unsupported_directives_are_refused_rather_than_ignored() {
        assert!(matches!(
            parse("#include \"other.vdf\"\n\"k\" \"v\""),
            Err(VdfError::UnsupportedDirective { .. })
        ));
    }

    #[test]
    fn nesting_is_bounded() {
        let mut text = String::from("\"root\"");
        for _ in 0..(MAX_DEPTH + 2) {
            text.push_str(" { \"n\"");
        }
        assert!(matches!(
            parse(&text),
            Err(VdfError::DepthLimitExceeded { .. })
        ));
    }

    #[test]
    fn non_ascii_values_survive_being_quoted() {
        // Fuzzer find: per-byte Latin-1 decoding split multi-byte characters on rewrite.
        let obj = parse("//+\no\n\u{310}").expect("must parse");
        let round_tripped = parse(&obj.to_string()).expect("must reparse");
        assert_eq!(obj, round_tripped);

        // The same thing in the shape it would really arrive in.
        let name = "Kerbal Space Program — Démo 日本語";
        let text = format!("\"name\"\t\t\"{name}\"\n");
        let parsed = parse(&text).expect("must parse");
        assert_eq!(parsed.get_str("name"), Some(name));
        assert_eq!(parsed.to_string(), text);
    }

    #[test]
    fn empty_input_is_an_empty_document() {
        assert_eq!(parse(""), Ok(Object::new()));
        assert_eq!(parse("// just a comment\n"), Ok(Object::new()));
    }
}
