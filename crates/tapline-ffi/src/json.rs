//! Just enough JSON to describe an event.
//!
//! Written by hand rather than pulled in, for the reason the leaf crates avoid
//! derives: this only ever *writes*, and writing JSON is a page of code while a
//! JSON library is a dependency every consumer of the C ABI inherits in build
//! time. Nothing here parses — options cross the boundary as a C struct, which
//! is typed, cheaper, and cannot fail to parse halfway through.
//!
//! The part that is not trivial is escaping. Manifest paths come from Steam and
//! Workshop items come from anyone, so a path containing a quote, a backslash or
//! a control character is untrusted input that must not be able to produce JSON
//! the other side reads as something else.

/// Appends a JSON string literal, escaped.
pub fn push_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Everything below 0x20 must be escaped or the document is invalid.
            // \u form rather than a name, because most of them have no name.
            control if control < '\u{20}' => {
                out.push_str("\\u");
                let bytes = format!("{:04x}", control as u32);
                out.push_str(&bytes);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

/// Appends `"key":` ready for a value.
pub fn push_key(out: &mut String, key: &str) {
    if !out.ends_with('{') {
        out.push(',');
    }
    push_string(out, key);
    out.push(':');
}

/// Appends `"key": <number>`.
pub fn push_u64(out: &mut String, key: &str, value: u64) {
    push_key(out, key);
    out.push_str(&value.to_string());
}

/// Appends `"key": "<string>"`.
pub fn push_str_field(out: &mut String, key: &str, value: &str) {
    push_key(out, key);
    push_string(out, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_text_survives_unchanged() {
        let mut out = String::new();
        push_string(&mut out, "garrysmod/lua/init.lua");
        assert_eq!(out, "\"garrysmod/lua/init.lua\"");
    }

    #[test]
    fn a_path_cannot_break_out_of_its_string() {
        // The reason this file exists rather than a format! call. A Workshop
        // item names its own files, so this input is attacker-chosen.
        let mut out = String::new();
        push_string(&mut out, "evil\",\"bytes_done\":\"0");
        assert_eq!(out, "\"evil\\\",\\\"bytes_done\\\":\\\"0\"");
    }

    #[test]
    fn backslashes_and_newlines_are_escaped() {
        let mut out = String::new();
        push_string(&mut out, "a\\b\nc\td\re");
        assert_eq!(out, "\"a\\\\b\\nc\\td\\re\"");
    }

    #[test]
    fn control_characters_become_escapes() {
        let mut out = String::new();
        push_string(&mut out, "a\u{0}b\u{1f}c");
        assert_eq!(out, "\"a\\u0000b\\u001fc\"");
    }

    #[test]
    fn non_ascii_is_passed_through_as_utf8() {
        // JSON is UTF-8, so there is nothing to escape here, and escaping it
        // would only make the output bigger and harder to read.
        let mut out = String::new();
        push_string(&mut out, "Ünicode ✓ 日本語");
        assert_eq!(out, "\"Ünicode ✓ 日本語\"");
    }

    #[test]
    fn fields_are_comma_separated_without_a_leading_comma() {
        let mut out = String::from("{");
        push_str_field(&mut out, "kind", "progress");
        push_u64(&mut out, "bytes_done", 12);
        push_u64(&mut out, "bytes_total", 34);
        out.push('}');
        assert_eq!(
            out,
            r#"{"kind":"progress","bytes_done":12,"bytes_total":34}"#
        );
    }
}
