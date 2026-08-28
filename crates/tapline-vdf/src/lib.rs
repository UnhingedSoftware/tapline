//! Valve's KeyValues text format; files must round-trip byte for byte.

mod parse;
mod write;

pub use parse::{VdfError, parse};

use std::fmt;

/// Nesting bound for hostile input; Valve's own files reach five or six levels.
pub const MAX_DEPTH: u32 = 64;

/// A KeyValues value; text KeyValues has no numeric types.
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

/// An ordered list of pairs; KeyValues permits duplicates and preserves order.
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

    /// The first value for `key`, case-insensitive like Valve's own reader.
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

    /// The first value parsed as unsigned; missing or non-numeric is `None`, not zero.
    #[must_use]
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get_str(key)?.trim().parse().ok()
    }

    /// Replaces the first value for `key` in place, or appends it if absent.
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
