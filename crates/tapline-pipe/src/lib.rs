//! A typed pipeline: source, decode, filter, sinks.
//!
//! ```no_run
//! # async fn example() -> Result<(), tapline_pipe::PipeError> {
//! # let mut session = tapline::Session::anonymous().await.unwrap();
//! use tapline_pipe::workshop;
//!
//! workshop(4000, 104_691_717)
//!     .gma()                        // bytes -> entries
//!     .only("lua/**")               // optional
//!     .zip("/srv/out.zip")          // a sink
//!     .dir("/srv/addons")           // and another, same pass
//!     .run(&mut session)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Why the types change as you chain
//!
//! [`Source`] has no `zip` method, because there is nothing to zip until the
//! bytes have been interpreted. `.gma()` is what turns a stream of bytes into a
//! stream of entries, and only then do the sinks appear. Writing the steps in a
//! nonsensical order is a compile error rather than a run-time one.
//!
//! # One pass
//!
//! Sinks tee. Asking for a directory and a zip reads the download once and
//! writes both, which is the thing the extension pipeline could not do — there,
//! `gmad,gmad-zip` read the finished archive twice.
//!
//! # The wire form
//!
//! A chain is sugar over [`Pipeline`], a plain value. That matters because the
//! chain cannot cross a C ABI: the JavaScript bindings build the same value and
//! send its text form, and an HTTP API would accept exactly that. The types are
//! for whoever is writing the code; the value is for whoever is transporting it.

#![forbid(unsafe_code)]

mod spec;

pub use spec::{Pipeline, Sink, SpecError};

use tapline::{InstallError, Session, Window};
use tapline_ids::{AppId, PublishedFileId};

/// Anything that can go wrong running a pipeline.
#[derive(Debug)]
pub enum PipeError {
    /// The download failed.
    Download(InstallError),
    /// The archive could not be read, or a sink refused.
    Archive(tapline_ext::ExtensionError),
    /// The pipeline itself was not usable.
    Spec(SpecError),
}

impl std::fmt::Display for PipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Download(error) => write!(f, "{error}"),
            Self::Archive(error) => write!(f, "{error}"),
            Self::Spec(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for PipeError {}

impl From<InstallError> for PipeError {
    fn from(error: InstallError) -> Self {
        Self::Download(error)
    }
}

impl From<tapline_ext::ExtensionError> for PipeError {
    fn from(error: tapline_ext::ExtensionError) -> Self {
        Self::Archive(error)
    }
}

impl From<SpecError> for PipeError {
    fn from(error: SpecError) -> Self {
        Self::Spec(error)
    }
}

/// What a pipeline produced.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Outcome {
    /// Entries written to each sink.
    pub entries: usize,
    /// Bytes fetched from the CDN.
    pub bytes_downloaded: u64,
    /// Bytes handed to the decoder.
    pub bytes_streamed: u64,
    /// The most chunks held back at once while reordering.
    pub peak_buffered: usize,
}

/// A Workshop item, before it has been given a meaning.
///
/// Has no sinks: there is nothing to write until the bytes are interpreted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    app: AppId,
    item: PublishedFileId,
    window: Window,
}

/// Starts a pipeline from a Workshop item.
#[must_use]
pub fn workshop(app: u32, item: u64) -> Source {
    Source {
        app: AppId(app),
        item: PublishedFileId(item),
        window: Window::default(),
    }
}

impl Source {
    /// How many chunks may be in flight while reordering. See [`Window`].
    #[must_use]
    pub const fn window(mut self, chunks: usize) -> Self {
        self.window = Window::new(chunks);
        self
    }

    /// Reads the download as a Garry's Mod addon.
    ///
    /// The step that turns bytes into entries, and the reason the sinks below
    /// exist at all.
    #[must_use]
    pub fn gma(self) -> Decoded {
        Decoded {
            source: self,
            pipeline: Pipeline::gma(),
        }
    }
}

/// A decoded stream, which can be filtered and written somewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    source: Source,
    pipeline: Pipeline,
}

impl Decoded {
    /// Keeps only entries matching a glob. Repeatable; any match selects.
    #[must_use]
    pub fn only(mut self, pattern: impl Into<String>) -> Self {
        self.pipeline.filters.push(pattern.into());
        self
    }

    /// Unpacks into a directory.
    #[must_use]
    pub fn dir(mut self, path: impl Into<String>) -> Piped {
        self.pipeline.sinks.push(Sink::Directory(path.into()));
        Piped {
            source: self.source,
            pipeline: self.pipeline,
        }
    }

    /// Writes a ZIP, deflating what gets smaller for it.
    #[must_use]
    pub fn zip(mut self, path: impl Into<String>) -> Piped {
        self.pipeline.sinks.push(Sink::Zip {
            path: path.into(),
            compress: true,
        });
        Piped {
            source: self.source,
            pipeline: self.pipeline,
        }
    }

    /// Writes a ZIP without deflating: roughly four times faster.
    #[must_use]
    pub fn zip_stored(mut self, path: impl Into<String>) -> Piped {
        self.pipeline.sinks.push(Sink::Zip {
            path: path.into(),
            compress: false,
        });
        Piped {
            source: self.source,
            pipeline: self.pipeline,
        }
    }
}

/// A pipeline with at least one sink, ready to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piped {
    source: Source,
    pipeline: Pipeline,
}

impl Piped {
    /// Adds another directory. Written in the same pass as the others.
    #[must_use]
    pub fn dir(mut self, path: impl Into<String>) -> Self {
        self.pipeline.sinks.push(Sink::Directory(path.into()));
        self
    }

    /// Adds another ZIP. Written in the same pass as the others.
    #[must_use]
    pub fn zip(mut self, path: impl Into<String>) -> Self {
        self.pipeline.sinks.push(Sink::Zip {
            path: path.into(),
            compress: true,
        });
        self
    }

    /// Adds a stored ZIP. Written in the same pass as the others.
    #[must_use]
    pub fn zip_stored(mut self, path: impl Into<String>) -> Self {
        self.pipeline.sinks.push(Sink::Zip {
            path: path.into(),
            compress: false,
        });
        self
    }

    /// The value this chain describes.
    ///
    /// What crosses a C ABI or an HTTP request; the chain is only how it is
    /// written in Rust.
    #[must_use]
    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    /// Runs it.
    pub async fn run(self, session: &mut Session) -> Result<Outcome, PipeError> {
        run_pipeline(
            session,
            self.source.app,
            self.source.item,
            self.source.window,
            &self.pipeline,
            &mut |_| {},
        )
        .await
    }

    /// Runs it, reporting progress.
    pub async fn run_observed(
        self,
        session: &mut Session,
        observe: &mut (dyn FnMut(tapline::Event) + Send),
    ) -> Result<Outcome, PipeError> {
        run_pipeline(
            session,
            self.source.app,
            self.source.item,
            self.source.window,
            &self.pipeline,
            observe,
        )
        .await
    }
}

/// Runs a pipeline value, however it was built.
///
/// The chain, the text form and any future HTTP request all end up here.
pub async fn run_pipeline(
    session: &mut Session,
    app: AppId,
    item: PublishedFileId,
    window: Window,
    pipeline: &Pipeline,
    observe: &mut (dyn FnMut(tapline::Event) + Send),
) -> Result<Outcome, PipeError> {
    let details = session
        .workshop_details(&[item])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| {
            PipeError::Download(InstallError::Io(format!(
                "Steam said nothing about item {item}"
            )))
        })?
        .map_err(|error| PipeError::Download(InstallError::Io(error.to_string())))?;
    let _ = app;

    // Every sink, fed from one pass over the download.
    let mut fanout = tapline_gmad::Fanout::new();
    for sink in &pipeline.sinks {
        fanout = fanout.with(sink.build()?);
    }

    let mut patterns = tapline_gmad::Patterns::all();
    for filter in &pipeline.filters {
        patterns = patterns.with(filter.clone());
    }

    let filtered = tapline_gmad::Filtered::new(fanout, patterns);
    let mut splitter = tapline_gmad::Splitter::new(filtered);

    let report = session
        .stream_workshop_item(
            &details,
            window,
            &mut |bytes| {
                splitter
                    .push(bytes)
                    .map_err(|error| InstallError::Io(error.to_string()))
            },
            observe,
        )
        .await?;

    let mut finished = splitter.finish()?;
    // Closes every sink, which is where a ZIP's central directory is written.
    tapline_gmad::EntrySink::finish(&mut finished)?;
    let entries = finished.passed();

    Ok(Outcome {
        entries,
        bytes_downloaded: report.bytes_downloaded,
        bytes_streamed: report.bytes_streamed,
        peak_buffered: report.peak_buffered,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_chain_builds_the_value_it_describes() {
        let piped = workshop(4000, 104_691_717)
            .gma()
            .only("lua/**")
            .zip("/srv/out.zip")
            .dir("/srv/addons");

        let pipeline = piped.pipeline();
        assert_eq!(pipeline.filters, vec!["lua/**".to_owned()]);
        assert_eq!(pipeline.sinks.len(), 2, "both sinks should be recorded");
        assert!(matches!(
            pipeline.sinks.first(),
            Some(Sink::Zip { compress: true, .. })
        ));
        assert!(matches!(pipeline.sinks.get(1), Some(Sink::Directory(_))));
    }

    #[test]
    fn filters_accumulate() {
        let piped = workshop(4000, 1)
            .gma()
            .only("lua/**")
            .only("*.txt")
            .dir("/x");
        assert_eq!(piped.pipeline().filters.len(), 2);
    }

    #[test]
    fn a_stored_zip_is_distinguishable_from_a_deflated_one() {
        let deflated = workshop(4000, 1).gma().zip("/a.zip");
        let stored = workshop(4000, 1).gma().zip_stored("/b.zip");
        assert!(matches!(
            deflated.pipeline().sinks.first(),
            Some(Sink::Zip { compress: true, .. })
        ));
        assert!(matches!(
            stored.pipeline().sinks.first(),
            Some(Sink::Zip {
                compress: false,
                ..
            })
        ));
    }

    #[test]
    fn the_window_is_carried_through() {
        let piped = workshop(4000, 1).window(4).gma().dir("/x");
        assert_eq!(piped.source.window.size, 4);
    }

    #[test]
    fn the_chain_round_trips_through_its_text_form() {
        // The property the C ABI depends on: what the chain built and what the
        // bindings send must be the same pipeline.
        let piped = workshop(4000, 1)
            .gma()
            .only("lua/**")
            .zip("/srv/out.zip")
            .dir("/srv/addons");
        let text = piped.pipeline().to_text();
        let parsed = Pipeline::parse(&text).expect("must parse");
        assert_eq!(&parsed, piped.pipeline());
    }
}
