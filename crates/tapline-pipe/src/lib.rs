//! A typed pipeline: source, decode, filter, sinks.
//!
//! ```no_run
//! # async fn example() -> Result<(), tapline_pipe::PipeError> {
//! use tapline_pipe::workshop;
//!
//! workshop(4000, 104_691_717)
//!     .gma()                        // bytes -> entries
//!     .only("lua/**")               // optional
//!     .zip("/srv/out.zip")          // where it goes; ends the chain
//!     .run()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! No session anywhere: one is taken from a pool and given back. Run several of
//! these at once and they get different sessions, never waiting on each other,
//! while still sharing one chunk budget. `run_with(&mut session)` is there for
//! anyone who wants to own it.
//!
//! # Why the types change as you chain
//!
//! [`Source`] has no `zip` method, because there is nothing to zip until the
//! bytes have been interpreted. `.gma()` is what turns a stream of bytes into a
//! stream of entries, and only then do the sinks appear. Writing the steps in a
//! nonsensical order is a compile error rather than a run-time one.
//!
//! # One way
//!
//! A stream has a direction, and choosing a destination ends the chain: there
//! is no `.zip(..).dir(..)`. Writing one download to two places is a fan-out,
//! which is a different thing with different costs — a second sink that buffers
//! multiplies what the first one holds — and `tapline_gmad::Fanout` is there
//! for anyone who wants it explicitly rather than by accident.
//!
//! # Modular by format
//!
//! `.gma()` is one [`tapline_ext::Decoder`]. The sinks, the filter and the
//! pipeline are written against [`tapline_ext::ArchiveEntry`] rather than
//! against any container, so a second format is a decoder and nothing else.
//!
//! Whether a format can be streamed at all is a property of the format: GMAD
//! works because its index comes first and its contents follow in index order.
//!
//! # The one-way rule is enforced, not just documented
//!
//! ```compile_fail
//! # use tapline_pipe::workshop;
//! // `Ready` has no sink methods, so this does not compile.
//! workshop(4000, 1).gma().zip("/out.zip").dir("/addons");
//! ```
//!
//! ```compile_fail
//! # use tapline_pipe::workshop;
//! // Nor does choosing a destination before saying what the bytes are.
//! workshop(4000, 1).zip("/out.zip");
//! ```
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

/// What is inside an archive, learned without downloading it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// Every entry the archive holds.
    pub entries: Vec<tapline_ext::ArchiveEntry>,
    /// The subset the pipeline's filters select. All of them when there are no
    /// filters.
    pub selected: Vec<tapline_ext::ArchiveEntry>,
    /// The archive's size.
    pub archive_bytes: u64,
    /// How much of it had to be read to learn all this.
    pub read_bytes: u64,
    /// What fetching the selection would transfer.
    pub selected_bytes: u64,
    /// What fetching the whole archive would transfer.
    pub total_bytes: u64,
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
        self.decode("gma")
    }

    /// Reads the download as a ZIP.
    ///
    /// Distinct from [`Decoded::zip`], which *writes* one — this says what the
    /// download already is. A ZIP keeps its index at the end, which is why this
    /// is possible at all: the archive is read by range rather than as a
    /// stream, so the tail is an ordinary read.
    #[must_use]
    pub fn zip(self) -> Decoded {
        self.decode("zip")
    }

    /// Reads the download as a named format.
    #[must_use]
    pub fn decode(self, format: impl Into<String>) -> Decoded {
        let mut pipeline = Pipeline::gma();
        pipeline.format = format.into();
        Decoded {
            source: self,
            pipeline,
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
    /// Lists what is inside, without downloading it.
    ///
    /// Reads only as much of the archive as the format needs to find its index
    /// — for a Garry's Mod addon that is the first 64 KiB, one chunk, whatever
    /// the archive's size. Measured on a real addon: 348 entries known after
    /// reading 65 KB of 8.7 MB.
    ///
    /// Filters apply, so this also answers "what would `only(..)` select, and
    /// what would fetching it cost".
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), tapline_pipe::PipeError> {
    /// let listing = tapline_pipe::workshop(4000, 104_691_717).gma().list().await?;
    /// for entry in &listing.entries {
    ///     println!("{} ({} bytes)", entry.path, entry.size);
    /// }
    /// println!("fetching the selection would cost {} bytes", listing.selected_bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(self) -> Result<Listing, PipeError> {
        let mut guard = tapline::SessionPool::shared().acquire().await?;
        let outcome = self.list_with(&mut guard).await;
        if outcome.is_err() {
            guard.poison();
        }
        outcome
    }

    /// Lists what is inside, on a session you own.
    pub async fn list_with(self, session: &mut Session) -> Result<Listing, PipeError> {
        let details = resolve(session, self.source.item).await?;
        let file = session.open_workshop_item(&details).await?;
        let entries = read_index(&file, &self.pipeline.format).await?;
        let selected = select(&entries, &self.pipeline)?;

        let ranges: Vec<(u64, u64)> = selected
            .iter()
            .map(|entry| (entry.offset, entry.stored_size))
            .collect();

        Ok(Listing {
            archive_bytes: file.len(),
            read_bytes: match index_location(&self.pipeline.format)? {
                tapline_ext::IndexLocation::Head(len) | tapline_ext::IndexLocation::Tail(len) => {
                    len.min(file.len())
                }
            },
            selected_bytes: file.cost_of(&ranges),
            total_bytes: file.cost_of(&[(0, file.len())]),
            selected,
            entries,
        })
    }

    /// Keeps only entries matching a glob. Repeatable; any match selects.
    ///
    /// A pattern matching nothing is not an error: asking what is there and
    /// finding nothing is a legitimate answer. Use [`Decoded::pick`] when you
    /// know the name.
    #[must_use]
    pub fn only(mut self, pattern: impl Into<String>) -> Self {
        self.pipeline.filters.push(pattern.into());
        self
    }

    /// Takes one named file, exactly.
    ///
    /// No pattern matching, so a file called `weapons/ak[47].lua` is asked for
    /// by that name rather than by something that happens to match it. Matching
    /// ignores case, because Workshop authors name files by hand.
    ///
    /// Unlike [`Decoded::only`], a name that is not in the archive is an
    /// **error**. A caller who named a file was making a claim about what is
    /// in there, and quietly producing an empty result would look like success.
    ///
    /// The natural companion to [`Decoded::list`]: list, choose, pick.
    #[must_use]
    pub fn pick(mut self, path: impl Into<String>) -> Self {
        self.pipeline.picks.push(path.into());
        self
    }

    /// Takes several named files.
    #[must_use]
    pub fn pick_all<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<String>,
    {
        self.pipeline
            .picks
            .extend(paths.into_iter().map(Into::into));
        self
    }

    /// Unpacks into a directory. Terminal.
    #[must_use]
    pub fn dir(mut self, path: impl Into<String>) -> Ready {
        self.pipeline.sink = Some(Sink::Directory(path.into()));
        Ready {
            source: self.source,
            pipeline: self.pipeline,
        }
    }

    /// Writes a ZIP, deflating what gets smaller for it. Terminal.
    #[must_use]
    pub fn zip(mut self, path: impl Into<String>) -> Ready {
        self.pipeline.sink = Some(Sink::Zip {
            path: path.into(),
            compress: true,
        });
        Ready {
            source: self.source,
            pipeline: self.pipeline,
        }
    }

    /// Writes a ZIP without deflating: roughly four times faster. Terminal.
    #[must_use]
    pub fn zip_stored(mut self, path: impl Into<String>) -> Ready {
        self.pipeline.sink = Some(Sink::Zip {
            path: path.into(),
            compress: false,
        });
        Ready {
            source: self.source,
            pipeline: self.pipeline,
        }
    }
}

/// A pipeline with its destination chosen, ready to run.
///
/// There is no second sink method here, and that is the point. A stream has a
/// direction: choosing where it goes ends the chain. Writing the same download
/// to two places is a fan-out, which has different costs — a second sink that
/// buffers would multiply what the first holds — and `tapline_gmad::Fanout` is
/// there for anyone who wants it explicitly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    source: Source,
    pipeline: Pipeline,
}

impl Ready {
    /// The value this chain describes.
    ///
    /// What crosses a C ABI or an HTTP request; the chain is only how it is
    /// written in Rust.
    #[must_use]
    pub const fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    /// Runs it on a pooled session.
    ///
    /// Nothing here needs to know a session exists. Concurrent runs get
    /// different sessions and never wait on each other, while sharing one chunk
    /// budget and one connection pool.
    pub async fn run(self) -> Result<Outcome, PipeError> {
        self.run_observed(&mut |_| {}).await
    }

    /// Runs it on a pooled session, reporting progress.
    pub async fn run_observed(
        self,
        observe: &mut (dyn FnMut(tapline::Event) + Send),
    ) -> Result<Outcome, PipeError> {
        let mut guard = tapline::SessionPool::shared().acquire().await?;
        let outcome = run_pipeline(
            &mut guard,
            self.source.app,
            self.source.item,
            self.source.window,
            &self.pipeline,
            observe,
        )
        .await;

        // A session that failed mid-operation may have a dead connection, and
        // the next caller inheriting it would fail for reasons that have
        // nothing to do with them.
        if outcome.is_err() {
            guard.poison();
        }
        outcome
    }

    /// Runs it on a session you own.
    ///
    /// The manual path. Everything the pool does — creating, reusing,
    /// heartbeating, discarding — becomes yours.
    pub async fn run_with(self, session: &mut Session) -> Result<Outcome, PipeError> {
        self.run_with_observed(session, &mut |_| {}).await
    }

    /// Runs it on a session you own, reporting progress.
    pub async fn run_with_observed(
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

/// Runs a pipeline by fetching only the entries it selects.
///
/// Possible because a depot file can be read by range: the index is fetched
/// first, the filter applied to it, and only the chunks holding the selected
/// entries are asked for. Measured on a real addon, `lua/**` selects 195 of 348
/// entries and costs 816 KB instead of 3.17 MB.
///
/// Used when the pipeline filters. Without a filter there is nothing to skip,
/// and streaming front to back is both simpler and no more expensive.
async fn run_selective(
    session: &mut Session,
    details: &tapline::WorkshopItem,
    pipeline: &Pipeline,
    observe: &mut (dyn FnMut(tapline::Event) + Send),
) -> Result<Outcome, PipeError> {
    let file = session.open_workshop_item(details).await?;
    let entries = read_index(&file, &pipeline.format).await?;
    let selected = select(&entries, pipeline)?;

    observe(tapline::Event::Planned {
        plan: tapline::Plan {
            download_bytes: file.cost_of(
                &selected
                    .iter()
                    .map(|entry| (entry.offset, entry.stored_size))
                    .collect::<Vec<_>>(),
            ),
            reused_bytes: 0,
            total_bytes: selected.iter().map(|entry| entry.size).sum(),
            file_count: selected.len() as u64,
            chunk_count: file.chunk_count() as u64,
        },
    });

    let mut sink = pipeline.sink.as_ref().ok_or(SpecError::NoSinks)?.build()?;

    // The whole index, not just the selection: a sink validating paths must see
    // every one, because an archive carrying an escaping path is hostile
    // whether or not this run wanted that file.
    sink.index(&entries)?;

    // The stored size, not the unpacked one: what is on the wire.
    let ranges: Vec<(u64, u64)> = selected
        .iter()
        .map(|entry| (entry.offset, entry.stored_size))
        .collect();
    let pieces = file.read_many(&ranges).await?;

    let mut streamed = 0_u64;
    for (index, (entry, stored)) in selected.iter().zip(pieces.iter()).enumerate() {
        let bytes = decode_entry(&pipeline.format, entry, stored)?;
        let bytes = &bytes;
        sink.begin(entry, index)?;
        if !bytes.is_empty() {
            sink.data(bytes)?;
        }
        sink.end()?;
        streamed += bytes.len() as u64;
        observe(tapline::Event::Progress {
            bytes_done: streamed,
            bytes_total: selected.iter().map(|entry| entry.size).sum(),
        });
    }
    sink.finish()?;

    Ok(Outcome {
        entries: selected.len(),
        bytes_downloaded: file.cost_of(&ranges),
        bytes_streamed: streamed,
        peak_buffered: 0,
    })
}

/// Runs a pipeline value, however it was built.
///
/// The chain, the text form and any future HTTP request all end up here.
/// Where a format keeps its index.
fn index_location(format: &str) -> Result<tapline_ext::IndexLocation, SpecError> {
    match format {
        "gma" => Ok(tapline_gmad::index_location()),
        "zip" => Ok(tapline_zip::index_location()),
        other => Err(SpecError::UnknownFormat(other.to_owned())),
    }
}

/// Reads a format's index, fetching whatever more it asks for.
///
/// The two-phase shape exists for ZIP: its central directory points at local
/// headers whose own lengths only those headers carry, so the data offsets are
/// one read further on. A GMAD answers in one phase and this loop runs once.
async fn read_index(
    file: &tapline::RemoteFile,
    format: &str,
) -> Result<Vec<tapline_ext::ArchiveEntry>, PipeError> {
    let window = match index_location(format)? {
        tapline_ext::IndexLocation::Head(len) => (0, len.min(file.len())),
        tapline_ext::IndexLocation::Tail(len) => {
            let len = len.min(file.len());
            (file.len().saturating_sub(len), len)
        }
    };
    let bytes = file.read(window.0, window.1).await?;

    let plan = match format {
        "gma" => tapline_ext::IndexPlan::done(tapline_gmad::plan(&bytes)?),
        "zip" => tapline_zip::plan(&bytes, window.0)?,
        other => return Err(SpecError::UnknownFormat(other.to_owned()).into()),
    };

    if plan.is_complete() {
        return Ok(plan.entries);
    }

    // The index itself was outside the window: fetch exactly what was asked
    // for and read it again.
    let plan = if plan.entries.is_empty() {
        let extra = file.read_many(&plan.needs).await?;
        let directory = extra.first().cloned().unwrap_or_default();
        match format {
            "zip" => tapline_zip::read_directory(&directory, usize::MAX)?,
            other => return Err(SpecError::UnknownFormat(other.to_owned()).into()),
        }
    } else {
        plan
    };

    let extra = file.read_many(&plan.needs).await?;
    match format {
        "zip" => Ok(tapline_zip::finalize(plan.entries, &extra)?),
        other => Err(SpecError::UnknownFormat(other.to_owned()).into()),
    }
}

/// Unpacks an entry's stored bytes for the format they came from.
fn decode_entry(
    format: &str,
    entry: &tapline_ext::ArchiveEntry,
    stored: &[u8],
) -> Result<Vec<u8>, PipeError> {
    match entry.compression {
        // Nothing was done to them, whatever the container.
        tapline_ext::Compression::Stored => Ok(stored.to_vec()),
        tapline_ext::Compression::Deflate => match format {
            "zip" => Ok(tapline_zip::decode(entry, stored)?),
            other => Err(SpecError::UnknownFormat(other.to_owned()).into()),
        },
    }
}

/// Which entries a pipeline takes, and why.
///
/// Globs and picks are a union: anything matching either is taken. A pick that
/// matches nothing fails here rather than at the end, so the caller is told
/// what is wrong before any content is fetched.
fn select(
    entries: &[tapline_ext::ArchiveEntry],
    pipeline: &Pipeline,
) -> Result<Vec<tapline_ext::ArchiveEntry>, SpecError> {
    let mut patterns = tapline_gmad::Patterns::all();
    for filter in &pipeline.filters {
        patterns = patterns.with(filter.clone());
    }

    for pick in &pipeline.picks {
        if !entries
            .iter()
            .any(|entry| entry.path.eq_ignore_ascii_case(pick))
        {
            return Err(SpecError::NoSuchEntry {
                path: pick.clone(),
                available: entries.len(),
            });
        }
    }

    // With picks and no globs, only the picks. With globs and no picks, only
    // the matches. With both, either.
    let take_all_patterns = pipeline.filters.is_empty() && !pipeline.picks.is_empty();
    Ok(entries
        .iter()
        .filter(|entry| {
            let picked = pipeline
                .picks
                .iter()
                .any(|pick| entry.path.eq_ignore_ascii_case(pick));
            let matched = !take_all_patterns && patterns.selects(&entry.path);
            picked || matched
        })
        .cloned()
        .collect())
}

/// Resolves a Workshop item to the details every path here needs.
async fn resolve(
    session: &mut Session,
    item: PublishedFileId,
) -> Result<tapline::WorkshopItem, PipeError> {
    session
        .workshop_details(&[item])
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| {
            PipeError::Download(InstallError::Io(format!(
                "Steam said nothing about item {item}"
            )))
        })?
        .map_err(|error| PipeError::Download(InstallError::Io(error.to_string())))
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
    let details = resolve(session, item).await?;
    let _ = app;

    pipeline.validate()?;

    // With a filter, fetch only what it selects. Without one there is nothing
    // to skip, and a front-to-back stream is simpler and costs the same.
    if pipeline.is_selective() {
        return run_selective(session, &details, pipeline, observe).await;
    }

    let sink = pipeline.sink.as_ref().ok_or(SpecError::NoSinks)?.build()?;

    let mut patterns = tapline_gmad::Patterns::all();
    for filter in &pipeline.filters {
        patterns = patterns.with(filter.clone());
    }

    let filtered = tapline_gmad::Filtered::new(sink, patterns);
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
        let ready = workshop(4000, 104_691_717)
            .gma()
            .only("lua/**")
            .zip("/srv/out.zip");

        let pipeline = ready.pipeline();
        assert_eq!(pipeline.filters, vec!["lua/**".to_owned()]);
        assert!(matches!(
            pipeline.sink,
            Some(Sink::Zip { compress: true, .. })
        ));
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
            deflated.pipeline().sink,
            Some(Sink::Zip { compress: true, .. })
        ));
        assert!(matches!(
            stored.pipeline().sink,
            Some(Sink::Zip {
                compress: false,
                ..
            })
        ));
    }

    #[test]
    fn the_window_is_carried_through() {
        let ready = workshop(4000, 1).window(4).gma().dir("/x");
        assert_eq!(ready.source.window.size, 4);
    }

    fn entry(path: &str, offset: u64, size: u64) -> tapline_ext::ArchiveEntry {
        tapline_ext::ArchiveEntry::stored(path.to_owned(), offset, size)
    }

    fn archive() -> Vec<tapline_ext::ArchiveEntry> {
        vec![
            entry("lua/a.lua", 0, 10),
            entry("lua/deep/b.lua", 10, 20),
            entry("materials/c.vmt", 30, 30),
            entry("weapons/ak[47].lua", 60, 40),
        ]
    }

    #[test]
    fn no_selection_takes_everything() {
        let pipeline = Pipeline::gma();
        let selected = select(&archive(), &pipeline).expect("select");
        assert_eq!(selected.len(), 4);
    }

    #[test]
    fn a_pick_takes_exactly_that_file() {
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("materials/c.vmt".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        assert_eq!(selected.len(), 1);
        assert_eq!(
            selected.first().map(|e| e.path.as_str()),
            Some("materials/c.vmt")
        );
    }

    #[test]
    fn a_pick_is_not_a_pattern() {
        // The reason picks exist. As a glob, the brackets would be a character
        // class and this file would be unreachable by its own name.
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("weapons/ak[47].lua".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn a_pick_that_is_not_there_is_an_error() {
        // Unlike a pattern. Naming a file is a claim about the archive, and a
        // silently empty result would look like success.
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("lua/nope.lua".to_owned());
        let error = select(&archive(), &pipeline).expect_err("must refuse");
        let text = error.to_string();
        assert!(text.contains("lua/nope.lua"), "{text}");
        assert!(text.contains("4 entries"), "{text}");
    }

    #[test]
    fn a_pattern_matching_nothing_is_not_an_error() {
        let mut pipeline = Pipeline::gma();
        pipeline.filters.push("nothing/**".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        assert!(selected.is_empty());
    }

    #[test]
    fn picks_and_patterns_are_a_union() {
        let mut pipeline = Pipeline::gma();
        pipeline.filters.push("materials/**".to_owned());
        pipeline.picks.push("lua/a.lua".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        let paths: Vec<_> = selected.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, vec!["lua/a.lua", "materials/c.vmt"]);
    }

    #[test]
    fn a_pick_ignores_case() {
        // Workshop authors name files by hand, and the glob matcher already
        // does the same.
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("LUA/A.LUA".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn picks_alone_do_not_pull_in_everything() {
        // With no globs the pattern set is empty, and an empty pattern set
        // matches everything — so a pick would have selected the whole archive
        // if the two were simply or-ed.
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("lua/a.lua".to_owned());
        assert_eq!(select(&archive(), &pipeline).expect("select").len(), 1);
    }

    #[test]
    fn the_chain_round_trips_through_its_text_form() {
        // The property the C ABI depends on: what the chain built and what the
        // bindings send must be the same pipeline.
        let ready = workshop(4000, 1)
            .gma()
            .only("lua/**")
            .pick("lua/exact.lua")
            .zip("/srv/out.zip");
        let text = ready.pipeline().to_text();
        let parsed = Pipeline::parse(&text).expect("must parse");
        assert_eq!(&parsed, ready.pipeline());
    }

    #[test]
    fn every_known_format_resolves_in_the_runner() {
        // `validate` checks a list and the runner dispatches with a `match`.
        // Nothing makes them agree except this, and when they disagreed the
        // symptom was a format that validated and then failed mid-run.
        for format in tapline_pipe_known_formats() {
            assert!(
                index_location(format).is_ok(),
                "{format} validates but the runner cannot locate its index"
            );
        }
    }

    /// The formats the spec says are usable.
    ///
    /// A function rather than the constant directly so the test reads as a
    /// question asked of the spec, which is the thing that could drift.
    fn tapline_pipe_known_formats() -> impl Iterator<Item = &'static str> {
        crate::spec::KNOWN_FORMATS.into_iter()
    }
}
