mod parse;
mod write;

pub use parse::{VdfError, parse};

use std::fmt;

pub const MAX_DEPTH: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Object(Object),
}

impl Value {
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::Object(_) => None,
        }
    }

    #[must_use]
    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Self::Object(o) => Some(o),
            Self::String(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object {
    pairs: Vec<(String, Value)>,
}

impl Object {
    #[must_use]
    pub const fn new() -> Self {
        Self { pairs: Vec::new() }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn push(&mut self, key: impl Into<String>, value: Value) {
        self.pairs.push((key.into(), value));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.pairs.iter().map(|(k, v)| (k.as_str(), v))
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }

    #[must_use]
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    #[must_use]
    pub fn get_object(&self, key: &str) -> Option<&Object> {
        self.get(key).and_then(Value::as_object)
    }

    #[must_use]
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get_str(key)?.trim().parse().ok()
    }

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

    pub fn set_str(&mut self, key: &str, value: impl Into<String>) {
        self.set(key, Value::String(value.into()));
    }
}

impl fmt::Display for Object {
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
