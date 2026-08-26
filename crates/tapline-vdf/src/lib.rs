//! Valve's KeyValues text format — what `appmanifest_*.acf` is written in.
//!
//! This matters more than a config parser normally would, because tapline is
//! expected to *share* state with the real Steam client and with steamcmd. An
//! install we produce has to be one they can read and update, and an install
//! they produced has to be one we can take over and patch incrementally. That
//! only works if a file survives a parse-and-write round trip byte for byte,
//! which is why the writer reproduces Valve's exact layout — tab indent, two
//! tabs between a key and its value, brace on its own line — rather than
//! whatever looks tidy.
//!
//! Ordering is preserved for the same reason: KeyValues is a list of pairs, not
//! a map. Duplicate keys are legal and Valve emits them, so nothing here
//! deduplicates.

mod parse;
mod write;

pub use parse::{VdfError, parse};

use std::fmt;

/// How deep nesting may go.
///
/// Valve's own files reach five or six levels. The limit is for hostile input:
/// KeyValues arrives from the network as well as from disk, and a file that is
/// nothing but open braces would otherwise recurse until the stack runs out.
pub const MAX_DEPTH: u32 = 64;

/// A KeyValues value: either a string or a nested object.
///
/// There are no numeric variants. Text KeyValues has no types — `"appid"
/// "232250"` is a string, and whether it means a number is the caller's
/// business. Inventing an integer variant here would mean guessing, and then
/// writing back a different spelling than we read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A leaf string.
    String(String),
    /// A nested block.
    Object(Object),
}

impl Value {
    /// The string, if this is one.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::Object(_) => None,
        }
    }

    /// The nested object, if this is one.
    #[must_use]
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Self::Object(o) => Some(o),
            Self::String(_) => None,
        }
    }
}

/// An ordered list of key/value pairs.
///
/// Deliberately not a map: KeyValues preserves order and permits duplicate keys,
/// and an install file that came back with its keys reordered would no longer
/// match what Steam wrote.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object {
    pairs: Vec<(String, Value)>,
}

impl Object {
    /// An empty object.
    #[must_use]
    pub const fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    /// How many pairs it holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Whether it holds no pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Appends a pair, keeping any existing key of the same name.
    pub fn push(&mut self, key: impl Into<String>, value: Value) {
        self.pairs.push((key.into(), value));
    }

    /// Every pair, in file order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.pairs.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// The first value for `key`.
    ///
    /// Case-insensitive, because Valve's own reader is: `appid` and `AppID`
    /// address the same field, and different Steam versions have written both.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }

    /// The first value for `key`, as a string.
    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    /// The first value for `key`, as an object.
    #[must_use]
    pub fn get_object(&self, key: &str) -> Option<&Object> {
        self.get(key).and_then(Value::as_object)
    }

    /// The first value for `key`, parsed as an unsigned integer.
    ///
    /// Returns `None` rather than zero when the field is missing or is not a
    /// number: a missing `size` and a `size` of 0 mean very different things to
    /// a downloader.
    #[must_use]
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get_str(key)?.trim().parse().ok()
    }

    /// Replaces the first value for `key`, or appends it if absent.
    ///
    /// Replacing in place is what keeps a rewritten `appmanifest` in the same
    /// field order Steam wrote it in.
    pub fn set(&mut self, key: &str, value: Value) {
        if let Some(slot) = self
            .pairs
            .iter_mut()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
        {
            slot.1 = value;
        } else {
            self.push(key.to_owned(), value);
        }
    }

    /// Replaces or appends a string value.
    pub fn set_str(&mut self, key: &str, value: impl Into<String>) {
        self.set(key, Value::String(value.into()));
    }
}

impl fmt::Display for Object {
    /// Writes the object in Valve's layout, ready to be saved as an ACF file.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write::write_object(f, self, 0)
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a str, &'a Value);
    type IntoIter = Box<dyn Iterator<Item = (&'a str, &'a Value)> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}
