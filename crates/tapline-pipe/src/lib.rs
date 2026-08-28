//! A typed pipeline: source, decode, filter, sinks.

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
    /// The subset the pipeline's filters select.
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
    #[must_use]
    pub fn gma(self) -> Decoded {
        self.decode("gma")
    }

    /// Reads the download as a ZIP.
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
    #[must_use]
    pub fn only(mut self, pattern: impl Into<String>) -> Self {
        self.pipeline.filters.push(pattern.into());
        self
    }

    /// Takes one named file, exactly; a name not in the archive is an error.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    source: Source,
    pipeline: Pipeline,
}

impl Ready {
    /// The value this chain describes.
    #[must_use]
    pub const fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    /// Runs it on a pooled session.
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

        if outcome.is_err() {
            guard.poison();
        }
        outcome
    }

    /// Runs it on a session you own.
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

async fn run_selective(
    session: &mut Session,
    details: &tapline::WorkshopItem,
    pipeline: &Pipeline,
    observe: &mut (dyn FnMut(tapline::Event) + Send),
) -> Result<Outcome, PipeError> {
    let file = session.open_workshop_item(details).await?;
    let entries = read_index(&file, &pipeline.format).await?;
    // Positions in the whole index: the sink resolves names against the full list.
    let chosen = select_indices(&entries, pipeline)?;
    let selected: Vec<tapline_ext::ArchiveEntry> = chosen
        .iter()
        .filter_map(|index| entries.get(*index).cloned())
        .collect();

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

    // Sinks validating paths must see every entry, selected or not.
    sink.index(&entries)?;

    let ranges: Vec<(u64, u64)> = selected
        .iter()
        .map(|entry| (entry.offset, entry.stored_size))
        .collect();
    let pieces = file.read_many(&ranges).await?;

    let mut streamed = 0_u64;
    for ((entry, stored), index) in selected.iter().zip(pieces.iter()).zip(chosen.iter()) {
        let bytes = decode_entry(&pipeline.format, entry, stored)?;
        let bytes = &bytes;
        sink.begin(entry, *index)?;
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

fn index_location(format: &str) -> Result<tapline_ext::IndexLocation, SpecError> {
    match format {
        "gma" => Ok(tapline_gmad::index_location()),
        "zip" => Ok(tapline_zip::index_location()),
        other => Err(SpecError::UnknownFormat(other.to_owned())),
    }
}

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

fn decode_entry(
    format: &str,
    entry: &tapline_ext::ArchiveEntry,
    stored: &[u8],
) -> Result<Vec<u8>, PipeError> {
    match entry.compression {
        tapline_ext::Compression::Stored => Ok(stored.to_vec()),
        tapline_ext::Compression::Deflate => match format {
            "zip" => Ok(tapline_zip::decode(entry, stored)?),
            other => Err(SpecError::UnknownFormat(other.to_owned()).into()),
        },
    }
}

/// Positions into the whole index: sinks resolve entry names by index against the full list.
fn select_indices(
    entries: &[tapline_ext::ArchiveEntry],
    pipeline: &Pipeline,
) -> Result<Vec<usize>, SpecError> {
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

    // An empty pattern set matches everything, so picks alone must not consult it.
    let take_all_patterns = pipeline.filters.is_empty() && !pipeline.picks.is_empty();
    Ok(entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            let picked = pipeline
                .picks
                .iter()
                .any(|pick| entry.path.eq_ignore_ascii_case(pick));
            let matched = !take_all_patterns && patterns.selects(&entry.path);
            picked || matched
        })
        .map(|(index, _)| index)
        .collect())
}

fn select(
    entries: &[tapline_ext::ArchiveEntry],
    pipeline: &Pipeline,
) -> Result<Vec<tapline_ext::ArchiveEntry>, SpecError> {
    Ok(select_indices(entries, pipeline)?
        .into_iter()
        .filter_map(|index| entries.get(index).cloned())
        .collect())
}

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
    fn a_selection_reports_positions_in_the_whole_index() {
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("materials/c.vmt".to_owned());
        let chosen = select_indices(&archive(), &pipeline).expect("select");
        assert_eq!(chosen, vec![2], "a pick must report where it is, not 0");
    }

    #[test]
    fn selected_positions_address_the_entries_they_came_from() {
        let entries = archive();
        let mut pipeline = Pipeline::gma();
        pipeline.filters.push("lua/**".to_owned());
        pipeline.picks.push("weapons/ak[47].lua".to_owned());

        let chosen = select_indices(&entries, &pipeline).expect("select");
        let selected = select(&entries, &pipeline).expect("select");
        assert_eq!(chosen.len(), selected.len());
        for (index, entry) in chosen.iter().zip(selected.iter()) {
            assert_eq!(
                entries.get(*index).map(|e| e.path.as_str()),
                Some(entry.path.as_str()),
                "position {index} does not address {}",
                entry.path
            );
        }
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
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("weapons/ak[47].lua".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn a_pick_that_is_not_there_is_an_error() {
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
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("LUA/A.LUA".to_owned());
        let selected = select(&archive(), &pipeline).expect("select");
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn picks_alone_do_not_pull_in_everything() {
        let mut pipeline = Pipeline::gma();
        pipeline.picks.push("lua/a.lua".to_owned());
        assert_eq!(select(&archive(), &pipeline).expect("select").len(), 1);
    }

    #[test]
    fn the_chain_round_trips_through_its_text_form() {
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
        for format in tapline_pipe_known_formats() {
            assert!(
                index_location(format).is_ok(),
                "{format} validates but the runner cannot locate its index"
            );
        }
    }

    fn tapline_pipe_known_formats() -> impl Iterator<Item = &'static str> {
        crate::spec::KNOWN_FORMATS.into_iter()
    }
}
