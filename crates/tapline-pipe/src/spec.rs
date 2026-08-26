//! The pipeline as a value, and its text form.
//!
//! A chain is nice to write and cannot cross a C ABI. This is what actually
//! travels: a small value the chain builds, the bindings encode, and the
//! runner consumes. An HTTP API would accept the same thing.
//!
//! The text form is line-based rather than JSON, because tapline writes JSON
//! and does not parse it — a parser here would be a parser to get wrong, on
//! input from outside the process. One directive per line, a keyword and the
//! rest of the line:
//!
//! ```text
//! decode gma
//! only lua/**
//! zip /srv/out.zip
//! dir /srv/addons
//! ```
//!
//! Paths are taken to the end of the line, so a path with spaces needs no
//! quoting and there is no quoting to get wrong. A path with a newline in it
//! cannot be expressed, which is the one thing this form gives up; such a path
//! would be refused by the filesystem layer anyway.

use tapline_ext::ExtensionError;

/// Why a pipeline could not be read or used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    /// A line was not understood.
    UnknownDirective {
        /// The keyword found.
        directive: String,
        /// Which line it was on, counting from one.
        line: usize,
    },
    /// A directive needed a value and had none.
    MissingValue {
        /// The keyword.
        directive: String,
        /// Which line.
        line: usize,
    },
    /// The decoder named is not one this build has.
    UnknownFormat(String),
    /// There was nothing to write to.
    NoSinks,
}

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownDirective { directive, line } => write!(
                f,
                "line {line}: unknown directive {directive:?}; known: decode, only, dir, zip, zip-stored"
            ),
            Self::MissingValue { directive, line } => {
                write!(f, "line {line}: {directive} needs a value")
            }
            Self::UnknownFormat(format) => {
                write!(f, "unknown format {format:?}; known: gma")
            }
            Self::NoSinks => write!(f, "the pipeline has no destination; add a dir or a zip"),
        }
    }
}

impl std::error::Error for SpecError {}

/// Where a pipeline writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sink {
    /// Unpack into a directory.
    Directory(String),
    /// Write a ZIP.
    Zip {
        /// Where.
        path: String,
        /// Whether to deflate entries that get smaller for it.
        compress: bool,
    },
}

impl Sink {
    /// Builds the thing that does the writing.
    pub(crate) fn build(&self) -> Result<Box<dyn tapline_gmad::EntrySink + Send>, ExtensionError> {
        match self {
            Self::Directory(path) => Ok(Box::new(tapline_gmad::ToDirectory::new(
                std::path::Path::new(path),
            ))),
            Self::Zip { path, compress } => Ok(Box::new(tapline_gmad::ZipSink::new(
                std::path::Path::new(path),
                *compress,
            )?)),
        }
    }
}

/// What to do with a download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    /// The format to read it as. Only `gma` today.
    pub format: String,
    /// Globs selecting entries. Empty selects everything.
    pub filters: Vec<String>,
    /// Where to write. At least one is required to run.
    pub sinks: Vec<Sink>,
}

impl Pipeline {
    /// A pipeline reading a Garry's Mod addon, with nothing configured yet.
    #[must_use]
    pub fn gma() -> Self {
        Self {
            format: "gma".to_owned(),
            filters: Vec::new(),
            sinks: Vec::new(),
        }
    }

    /// Checks the pipeline can run.
    pub fn validate(&self) -> Result<(), SpecError> {
        if self.format != "gma" {
            return Err(SpecError::UnknownFormat(self.format.clone()));
        }
        if self.sinks.is_empty() {
            return Err(SpecError::NoSinks);
        }
        Ok(())
    }

    /// The text form.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = format!("decode {}\n", self.format);
        for filter in &self.filters {
            out.push_str(&format!("only {filter}\n"));
        }
        for sink in &self.sinks {
            match sink {
                Sink::Directory(path) => out.push_str(&format!("dir {path}\n")),
                Sink::Zip {
                    path,
                    compress: true,
                } => out.push_str(&format!("zip {path}\n")),
                Sink::Zip {
                    path,
                    compress: false,
                } => out.push_str(&format!("zip-stored {path}\n")),
            }
        }
        out
    }

    /// Reads the text form.
    pub fn parse(text: &str) -> Result<Self, SpecError> {
        let mut pipeline = Self {
            format: "gma".to_owned(),
            filters: Vec::new(),
            sinks: Vec::new(),
        };

        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let trimmed = raw.trim();
            // Blank lines and comments, so a pipeline can be kept in a file a
            // person edits.
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let (directive, rest) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
            let value = rest.trim();
            let need = |value: &str| -> Result<String, SpecError> {
                if value.is_empty() {
                    Err(SpecError::MissingValue {
                        directive: directive.to_owned(),
                        line,
                    })
                } else {
                    Ok(value.to_owned())
                }
            };

            match directive {
                "decode" => pipeline.format = need(value)?,
                "only" => pipeline.filters.push(need(value)?),
                "dir" => pipeline.sinks.push(Sink::Directory(need(value)?)),
                "zip" => pipeline.sinks.push(Sink::Zip {
                    path: need(value)?,
                    compress: true,
                }),
                "zip-stored" => pipeline.sinks.push(Sink::Zip {
                    path: need(value)?,
                    compress: false,
                }),
                other => {
                    return Err(SpecError::UnknownDirective {
                        directive: other.to_owned(),
                        line,
                    });
                }
            }
        }

        Ok(pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pipeline_round_trips() {
        let original = Pipeline {
            format: "gma".to_owned(),
            filters: vec!["lua/**".to_owned(), "*.txt".to_owned()],
            sinks: vec![
                Sink::Directory("/srv/addons".to_owned()),
                Sink::Zip {
                    path: "/srv/out.zip".to_owned(),
                    compress: true,
                },
                Sink::Zip {
                    path: "/srv/fast.zip".to_owned(),
                    compress: false,
                },
            ],
        };
        assert_eq!(Pipeline::parse(&original.to_text()), Ok(original));
    }

    #[test]
    fn a_path_with_spaces_needs_no_quoting() {
        // The reason the value is the rest of the line rather than a token.
        let pipeline = Pipeline::parse("decode gma\ndir /srv/my addons/here\n").expect("parse");
        assert_eq!(
            pipeline.sinks,
            vec![Sink::Directory("/srv/my addons/here".to_owned())]
        );
    }

    #[test]
    fn blank_lines_and_comments_are_ignored() {
        let pipeline =
            Pipeline::parse("# what to do\n\ndecode gma\n\n  # a filter\n  only lua/**\ndir /x\n")
                .expect("parse");
        assert_eq!(pipeline.filters, vec!["lua/**".to_owned()]);
        assert_eq!(pipeline.sinks.len(), 1);
    }

    #[test]
    fn an_unknown_directive_names_itself_and_its_line() {
        let error = Pipeline::parse("decode gma\nexplode /x\n").expect_err("must refuse");
        let text = error.to_string();
        assert!(text.contains("line 2"), "{text}");
        assert!(text.contains("explode"), "{text}");
        assert!(text.contains("known:"), "{text}");
    }

    #[test]
    fn a_directive_without_a_value_is_refused() {
        // Silently accepting `dir` with no path would write to whatever the
        // empty path resolves to.
        let error = Pipeline::parse("decode gma\ndir\n").expect_err("must refuse");
        assert!(error.to_string().contains("needs a value"), "{error}");
    }

    #[test]
    fn a_pipeline_with_no_sinks_does_not_validate() {
        let pipeline = Pipeline::parse("decode gma\nonly lua/**\n").expect("parse");
        assert_eq!(pipeline.validate(), Err(SpecError::NoSinks));
    }

    #[test]
    fn an_unknown_format_is_refused_by_name() {
        let pipeline = Pipeline::parse("decode rar\ndir /x\n").expect("parse");
        assert!(matches!(
            pipeline.validate(),
            Err(SpecError::UnknownFormat(_))
        ));
    }

    #[test]
    fn an_empty_text_parses_to_something_that_does_not_validate() {
        // Rather than to something that runs and writes nowhere.
        let pipeline = Pipeline::parse("").expect("parse");
        assert_eq!(pipeline.validate(), Err(SpecError::NoSinks));
    }

    #[test]
    fn sink_order_is_preserved() {
        // The order sinks were added is the order they are written, which is
        // what a caller asked for even if it does not matter to the result.
        let text = "decode gma\nzip /a.zip\ndir /b\nzip /c.zip\n";
        let pipeline = Pipeline::parse(text).expect("parse");
        assert!(matches!(pipeline.sinks.first(), Some(Sink::Zip { .. })));
        assert!(matches!(pipeline.sinks.get(1), Some(Sink::Directory(_))));
        assert!(matches!(pipeline.sinks.get(2), Some(Sink::Zip { .. })));
        assert_eq!(pipeline.to_text(), text);
    }
}
