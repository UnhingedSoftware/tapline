//! The KeyValues writer.
//!
//! This reproduces Valve's layout exactly rather than picking a tidier one,
//! because the files it writes are shared with the Steam client, steamcmd and
//! every host panel that has ever grepped an `appmanifest`. A file that came
//! back reformatted would still parse, and would still show up as a diff to
//! everyone watching those files for changes.
//!
//! Valve's layout is:
//!
//! ```text
//! "AppState"
//! {
//! →   "appid"→→"232250"
//! →   "InstalledDepots"
//! →   {
//! →   →   "232251"
//! →   →   {
//! →   →   →   "manifest"→→"3005584029853244745"
//! →   →   }
//! →   }
//! }
//! ```
//!
//! — tab per level of indent, two tabs between a key and its string value, and
//! a nested block's brace on its own line at the key's indent.

use crate::{Object, Value};
use std::fmt;

/// Writes `object`'s pairs at `depth` levels of indentation.
pub(crate) fn write_object(
    f: &mut fmt::Formatter<'_>,
    object: &Object,
    depth: usize,
) -> fmt::Result {
    for (key, value) in object.iter() {
        write_indent(f, depth)?;
        write_quoted(f, key)?;

        match value {
            Value::String(s) => {
                // Two tabs, which is what Valve emits regardless of key length.
                f.write_str("\t\t")?;
                write_quoted(f, s)?;
                f.write_str("\n")?;
            }
            Value::Object(nested) => {
                f.write_str("\n")?;
                write_indent(f, depth)?;
                f.write_str("{\n")?;
                write_object(f, nested, depth + 1)?;
                write_indent(f, depth)?;
                f.write_str("}\n")?;
            }
        }
    }
    Ok(())
}

fn write_indent(f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        f.write_str("\t")?;
    }
    Ok(())
}

/// Writes a quoted string, escaping only what would otherwise break the quoting.
///
/// Backslash and double quote are escaped. Tabs and newlines inside a value are
/// left as literal bytes, which is what Valve writes and what its reader accepts
/// — escaping them would produce a file that differs from the one Steam wrote
/// for the same content.
fn write_quoted(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    for ch in s.chars() {
        match ch {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            other => f.write_fmt(format_args!("{other}"))?,
        }
    }
    f.write_str("\"")
}

#[cfg(test)]
mod tests {
    use crate::{Object, Value, parse};

    /// A real `appmanifest_232250.acf` in Valve's own layout, trimmed to the
    /// fields that matter. This is the round-trip fixture the M1 gate names.
    const ACF: &str = "\"AppState\"\n\
{\n\
\t\"appid\"\t\t\"232250\"\n\
\t\"Universe\"\t\t\"1\"\n\
\t\"name\"\t\t\"Team Fortress 2 Dedicated Server\"\n\
\t\"StateFlags\"\t\t\"4\"\n\
\t\"installdir\"\t\t\"Team Fortress 2 Dedicated Server\"\n\
\t\"LastUpdated\"\t\t\"1756089600\"\n\
\t\"SizeOnDisk\"\t\t\"16106127360\"\n\
\t\"buildid\"\t\t\"17442188\"\n\
\t\"InstalledDepots\"\n\
\t{\n\
\t\t\"232251\"\n\
\t\t{\n\
\t\t\t\"manifest\"\t\t\"3005584029853244745\"\n\
\t\t\t\"size\"\t\t\"16106127360\"\n\
\t\t}\n\
\t}\n\
\t\"UserConfig\"\n\
\t{\n\
\t\t\"language\"\t\t\"english\"\n\
\t}\n\
}\n";

    #[test]
    fn an_acf_file_round_trips_byte_for_byte() {
        // The M1 gate. If this ever fails, tapline can no longer share an
        // install directory with the Steam client without rewriting files it
        // did not mean to change.
        let parsed = parse(ACF).expect("the fixture must parse");
        assert_eq!(parsed.to_string(), ACF);
    }

    #[test]
    fn round_trip_is_stable_under_repetition() {
        let once = parse(ACF).expect("must parse").to_string();
        let twice = parse(&once).expect("must reparse").to_string();
        assert_eq!(once, twice);
    }

    #[test]
    fn editing_a_field_preserves_every_other_byte() {
        // Updating an install must rewrite the fields that changed and nothing
        // else — a reordered or reformatted file is a diff for everyone
        // watching appmanifests.
        let mut parsed = parse(ACF).expect("must parse");
        let state = match parsed.get("AppState") {
            Some(Value::Object(o)) => o.clone(),
            _ => panic!("AppState must be an object"),
        };
        let mut state = state;
        state.set_str("buildid", "17999999");
        parsed.set("AppState", Value::Object(state));

        let written = parsed.to_string();
        assert_eq!(written, ACF.replace("17442188", "17999999"));
    }

    #[test]
    fn quotes_and_backslashes_survive_a_round_trip() {
        let mut obj = Object::new();
        obj.push("path", Value::String("C:\\srv\\tf2".into()));
        obj.push("quoted", Value::String("say \"hello\"".into()));

        let reparsed = parse(&obj.to_string()).expect("must reparse");
        assert_eq!(reparsed.get_str("path"), Some("C:\\srv\\tf2"));
        assert_eq!(reparsed.get_str("quoted"), Some("say \"hello\""));
    }

    #[test]
    fn empty_nested_blocks_survive() {
        // A depot list with no depots is a legal state, and it must not
        // disappear on rewrite.
        let mut obj = Object::new();
        obj.push("InstalledDepots", Value::Object(Object::new()));
        let text = obj.to_string();
        assert_eq!(text, "\"InstalledDepots\"\n{\n}\n");
        assert_eq!(parse(&text).expect("must reparse"), obj);
    }
}
