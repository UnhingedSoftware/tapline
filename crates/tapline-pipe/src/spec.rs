use tapline_ext::ExtensionError;

pub const KNOWN_FORMATS: [&str; 2] = ["gma", "zip"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    UnknownDirective { directive: String, line: usize },
    MissingValue { directive: String, line: usize },
    UnknownFormat(String),
    NoSinks,
    NoSuchEntry { path: String, available: usize },
}

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownDirective { directive, line } => write!(
                f,
                "line {line}: unknown directive {directive:?}; \
                 known: decode, only, pick, dir, zip, zip-stored"
            ),
            Self::MissingValue { directive, line } => {
                write!(f, "line {line}: {directive} needs a value")
            }
            Self::UnknownFormat(format) => {
                write!(
                    f,
                    "unknown format {format:?}; known: {}",
                    KNOWN_FORMATS.join(", ")
                )
            }
            Self::NoSinks => write!(f, "the pipeline has no destination; add a dir or a zip"),
            Self::NoSuchEntry { path, available } => write!(
                f,
                "the archive has no entry {path:?}; it has {available} entries — \
                 list it first, or use `only` for a pattern that may match nothing"
            ),
        }
    }
}

impl std::error::Error for SpecError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sink {
    Directory(String),
    Zip { path: String, compress: bool },
}

impl Sink {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    pub format: String,
    pub filters: Vec<String>,
    pub picks: Vec<String>,
    pub sink: Option<Sink>,
}

impl Pipeline {
    #[must_use]
    pub fn gma() -> Self {
        Self {
            format: "gma".to_owned(),
            filters: Vec::new(),
            picks: Vec::new(),
            sink: None,
        }
    }

    #[must_use]
    pub fn is_selective(&self) -> bool {
        !self.filters.is_empty() || !self.picks.is_empty()
    }

    pub fn validate(&self) -> Result<(), SpecError> {
        if !KNOWN_FORMATS.contains(&self.format.as_str()) {
            return Err(SpecError::UnknownFormat(self.format.clone()));
        }
        if self.sink.is_none() {
            return Err(SpecError::NoSinks);
        }
        Ok(())
    }

    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = format!("decode {}\n", self.format);
        for filter in &self.filters {
            out.push_str(&format!("only {filter}\n"));
        }
        for pick in &self.picks {
            out.push_str(&format!("pick {pick}\n"));
        }
        for sink in self.sink.iter() {
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

    pub fn parse(text: &str) -> Result<Self, SpecError> {
        let mut pipeline = Self {
            format: "gma".to_owned(),
            filters: Vec::new(),
            picks: Vec::new(),
            sink: None,
        };

        for (index, raw) in text.lines().enumerate() {
            let line = index + 1;
            let trimmed = raw.trim();
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
                "pick" => pipeline.picks.push(need(value)?),
                "dir" => pipeline.sink = Some(Sink::Directory(need(value)?)),
                "zip" => {
                    pipeline.sink = Some(Sink::Zip {
                        path: need(value)?,
                        compress: true,
                    });
                }
                "zip-stored" => {
                    pipeline.sink = Some(Sink::Zip {
                        path: need(value)?,
                        compress: false,
                    });
                }
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
            picks: vec!["lua/autorun/init.lua".to_owned()],
            sink: Some(Sink::Zip {
                path: "/srv/out.zip".to_owned(),
                compress: true,
            }),
        };
        assert_eq!(Pipeline::parse(&original.to_text()), Ok(original));
    }

    #[test]
    fn a_second_destination_replaces_the_first() {
        let pipeline = Pipeline::parse("decode gma\ndir /a\nzip /b.zip\n").expect("parse");
        assert_eq!(
            pipeline.sink,
            Some(Sink::Zip {
                path: "/b.zip".to_owned(),
                compress: true
            })
        );
    }

    #[test]
    fn a_path_with_spaces_needs_no_quoting() {
        let pipeline = Pipeline::parse("decode gma\ndir /srv/my addons/here\n").expect("parse");
        assert_eq!(
            pipeline.sink,
            Some(Sink::Directory("/srv/my addons/here".to_owned()))
        );
    }

    #[test]
    fn blank_lines_and_comments_are_ignored() {
        let pipeline =
            Pipeline::parse("# what to do\n\ndecode gma\n\n  # a filter\n  only lua/**\ndir /x\n")
                .expect("parse");
        assert_eq!(pipeline.filters, vec!["lua/**".to_owned()]);
        assert!(pipeline.sink.is_some());
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
    fn every_known_format_validates() {
        for format in KNOWN_FORMATS {
            let pipeline = Pipeline::parse(&format!("decode {format}\ndir /x\n")).expect("parse");
            assert_eq!(
                pipeline.validate(),
                Ok(()),
                "{format} is a known format and did not validate"
            );
        }
    }

    #[test]
    fn the_refusal_lists_the_formats_that_would_have_worked() {
        let pipeline = Pipeline::parse("decode rar\ndir /x\n").expect("parse");
        let text = pipeline.validate().expect_err("must refuse").to_string();
        for format in KNOWN_FORMATS {
            assert!(text.contains(format), "{text} does not mention {format}");
        }
    }

    #[test]
    fn an_empty_text_parses_to_something_that_does_not_validate() {
        let pipeline = Pipeline::parse("").expect("parse");
        assert_eq!(pipeline.validate(), Err(SpecError::NoSinks));
    }

    #[test]
    fn a_stored_zip_survives_the_round_trip() {
        let text = "decode gma\nzip-stored /fast.zip\n";
        let pipeline = Pipeline::parse(text).expect("parse");
        assert_eq!(
            pipeline.sink,
            Some(Sink::Zip {
                path: "/fast.zip".to_owned(),
                compress: false
            })
        );
        assert_eq!(pipeline.to_text(), text);
    }
}
