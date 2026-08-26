//! Tokenizer for `.proto` files.
//!
//! Scoped to what Valve's schema actually contains, which was measured rather
//! than assumed: no `map<>`, no groups, and no `syntax` declaration anywhere, so
//! every file is proto2 by omission.

use std::fmt;

/// One token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// An identifier, possibly dotted: `optional`, `ContentManifestPayload`,
    /// `.CMsgClientLogon`.
    Ident(String),
    /// An integer literal, with its sign.
    ///
    /// Held as `i128` rather than `i64` because Valve uses `18446744073709551615`
    /// — `u64::MAX` — as the "no job" sentinel default on
    /// `CMsgProtoBufHeader.jobid_source`, and that is the field the whole RPC
    /// correlation scheme hangs off. An `i64` token type parses the entire
    /// schema except the one number that matters most.
    Int(i128),
    /// A quoted string, unescaped.
    Str(String),
    /// One of `{ } [ ] ( ) < > = ; , .`
    Punct(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident(s) => f.write_str(s),
            Self::Int(i) => write!(f, "{i}"),
            Self::Str(s) => write!(f, "{s:?}"),
            Self::Punct(c) => write!(f, "{c}"),
        }
    }
}

/// A token with the line it came from, so an error can say where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned {
    /// The token.
    pub token: Token,
    /// 1-based line number.
    pub line: u32,
}

/// A tokenizer failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    /// What went wrong.
    pub message: String,
    /// 1-based line number.
    pub line: u32,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Splits `input` into tokens.
pub fn lex(input: &str) -> Result<Vec<Spanned>, LexError> {
    let bytes = input.as_bytes();
    let mut pos = 0_usize;
    let mut line = 1_u32;
    let mut out = Vec::new();

    while pos < bytes.len() {
        let Some(&byte) = bytes.get(pos) else { break };

        // Whitespace.
        if byte.is_ascii_whitespace() {
            if byte == b'\n' {
                line += 1;
            }
            pos += 1;
            continue;
        }

        // Comments: `//` to end of line, `/* */` to the closing marker.
        if byte == b'/' {
            match bytes.get(pos + 1) {
                Some(b'/') => {
                    while pos < bytes.len() && bytes.get(pos) != Some(&b'\n') {
                        pos += 1;
                    }
                    continue;
                }
                Some(b'*') => {
                    pos += 2;
                    loop {
                        match (bytes.get(pos), bytes.get(pos + 1)) {
                            (Some(b'*'), Some(b'/')) => {
                                pos += 2;
                                break;
                            }
                            (Some(b'\n'), _) => {
                                line += 1;
                                pos += 1;
                            }
                            (Some(_), _) => pos += 1,
                            (None, _) => {
                                return Err(LexError {
                                    message: "unterminated block comment".into(),
                                    line,
                                });
                            }
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }

        // Strings.
        if byte == b'"' || byte == b'\'' {
            let quote = byte;
            pos += 1;
            let mut value = Vec::new();
            loop {
                let Some(&b) = bytes.get(pos) else {
                    return Err(LexError {
                        message: "unterminated string".into(),
                        line,
                    });
                };
                pos += 1;
                match b {
                    b if b == quote => break,
                    b'\\' => {
                        let Some(&escaped) = bytes.get(pos) else {
                            return Err(LexError {
                                message: "unterminated escape".into(),
                                line,
                            });
                        };
                        pos += 1;
                        value.push(match escaped {
                            b'n' => b'\n',
                            b't' => b'\t',
                            b'r' => b'\r',
                            other => other,
                        });
                    }
                    b'\n' => {
                        return Err(LexError {
                            message: "newline inside string".into(),
                            line,
                        });
                    }
                    other => value.push(other),
                }
            }
            out.push(Spanned {
                token: Token::Str(String::from_utf8_lossy(&value).into_owned()),
                line,
            });
            continue;
        }

        // Numbers. A leading `-` only starts one if a digit follows, so that
        // `-` in an option value is not mistaken for a negative literal.
        let numeric_start = byte.is_ascii_digit()
            || (byte == b'-' && bytes.get(pos + 1).is_some_and(u8::is_ascii_digit));
        if numeric_start {
            let start = pos;
            if byte == b'-' {
                pos += 1;
            }
            let hex = bytes.get(pos) == Some(&b'0')
                && matches!(bytes.get(pos + 1), Some(b'x') | Some(b'X'));
            if hex {
                pos += 2;
                while bytes.get(pos).is_some_and(u8::is_ascii_hexdigit) {
                    pos += 1;
                }
            } else {
                while bytes
                    .get(pos)
                    .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
                {
                    pos += 1;
                }
            }
            let text = String::from_utf8_lossy(bytes.get(start..pos).unwrap_or_default());

            let parsed = if hex {
                let digits = text.trim_start_matches("0x").trim_start_matches("0X");
                i128::from_str_radix(digits, 16).ok()
            } else if text.contains('.') {
                // A float literal. Only ever appears as an option value, which
                // the parser discards, so the exact value does not matter.
                Some(0)
            } else {
                text.parse::<i128>().ok()
            };

            let Some(value) = parsed else {
                return Err(LexError {
                    message: format!("could not parse number {text}"),
                    line,
                });
            };
            out.push(Spanned {
                token: Token::Int(value),
                line,
            });
            continue;
        }

        // Identifiers, which may be dotted or start with a dot for a
        // fully-qualified type reference.
        if byte.is_ascii_alphabetic() || byte == b'_' || byte == b'.' {
            // A lone `.` that is not followed by an identifier character is
            // punctuation, not the start of a qualified name.
            let is_qualified_name = byte != b'.'
                || bytes
                    .get(pos + 1)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
            if is_qualified_name {
                let start = pos;
                pos += 1;
                while bytes
                    .get(pos)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'.')
                {
                    pos += 1;
                }
                out.push(Spanned {
                    token: Token::Ident(
                        String::from_utf8_lossy(bytes.get(start..pos).unwrap_or_default())
                            .into_owned(),
                    ),
                    line,
                });
                continue;
            }
        }

        out.push(Spanned {
            token: Token::Punct(char::from(byte)),
            line,
        });
        pos += 1;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(input: &str) -> Vec<Token> {
        lex(input)
            .expect("must lex")
            .into_iter()
            .map(|s| s.token)
            .collect()
    }

    #[test]
    fn lexes_a_field_declaration() {
        assert_eq!(
            tokens("optional uint32 depot_id = 1;"),
            vec![
                Token::Ident("optional".into()),
                Token::Ident("uint32".into()),
                Token::Ident("depot_id".into()),
                Token::Punct('='),
                Token::Int(1),
                Token::Punct(';'),
            ]
        );
    }

    #[test]
    fn qualified_type_names_stay_one_token() {
        assert_eq!(
            tokens("repeated .ContentManifestPayload.FileMapping.ChunkData chunks = 6;")
                .first()
                .cloned(),
            Some(Token::Ident("repeated".into()))
        );
        assert_eq!(
            tokens(".A.B.C x = 1;").first().cloned(),
            Some(Token::Ident(".A.B.C".into()))
        );
    }

    #[test]
    fn comments_are_dropped() {
        assert_eq!(
            tokens("// leading\n/* block\n spanning */ optional // trailing\n"),
            vec![Token::Ident("optional".into())]
        );
    }

    #[test]
    fn negative_and_hex_literals_are_read() {
        assert_eq!(tokens("= -1;").get(1).cloned(), Some(Token::Int(-1)));
        assert_eq!(
            tokens("= 0x8000;").get(1).cloned(),
            Some(Token::Int(0x8000))
        );
    }

    #[test]
    fn strings_are_unescaped() {
        // `[` `(` `description` `)` `=` `"..."` `]`
        assert_eq!(
            tokens(r#"[(description) = "a \"quoted\" thing"]"#)
                .get(5)
                .cloned(),
            Some(Token::Str("a \"quoted\" thing".into()))
        );
    }

    #[test]
    fn the_job_id_sentinel_survives() {
        // u64::MAX is the "no job" default on CMsgProtoBufHeader.jobid_source.
        // An i64 token type loses exactly this value, and it is the one the RPC
        // correlation scheme depends on.
        assert_eq!(
            tokens("= 18446744073709551615;").get(1).cloned(),
            Some(Token::Int(18_446_744_073_709_551_615))
        );
    }

    #[test]
    fn unterminated_constructs_are_errors_not_silent_truncation() {
        assert!(lex("\"unterminated").is_err());
        assert!(lex("/* unterminated").is_err());
    }

    #[test]
    fn line_numbers_track_through_comments_and_strings() {
        let spans = lex("a\n// comment\nb\n/* two\nlines */\nc").expect("must lex");
        let lines: Vec<u32> = spans.iter().map(|s| s.line).collect();
        assert_eq!(lines, vec![1, 3, 6]);
    }
}
