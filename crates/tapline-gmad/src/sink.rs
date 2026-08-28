use crate::zip;
use std::io::Write;
use std::path::{Path, PathBuf};
use tapline_ext::ExtensionError;
use tapline_ext::{ArchiveEntry, EntrySink};

fn validate(entries: &[ArchiveEntry], root: &Path) -> Result<Vec<PathBuf>, ExtensionError> {
    entries
        .iter()
        .map(|entry| {
            tapline_fs::validate_path(&entry.path)
                .map(|safe| safe.resolve(root))
                .map_err(|reason| ExtensionError::UnsafePath {
                    path: entry.path.clone(),
                    reason: reason.to_string(),
                })
        })
        .collect()
}

pub struct ToDirectory {
    dest: PathBuf,
    targets: Vec<PathBuf>,
    names: Vec<String>,
    current: Option<std::io::BufWriter<std::fs::File>>,
    at: usize,
    produced: Vec<String>,
}

impl ToDirectory {
    #[must_use]
    pub fn new(dest: &Path) -> Self {
        Self {
            dest: dest.to_path_buf(),
            targets: Vec::new(),
            names: Vec::new(),
            current: None,
            at: 0,
            produced: Vec::new(),
        }
    }

    #[must_use]
    pub fn produced(&self) -> &[String] {
        &self.produced
    }

    #[must_use]
    pub fn into_produced(self) -> Vec<String> {
        self.produced
    }
}

impl EntrySink for ToDirectory {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
        self.targets = validate(entries, &self.dest)?;
        self.names = entries.iter().map(|entry| entry.path.clone()).collect();
        Ok(())
    }

    fn begin(&mut self, _entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
        self.at = index;
        let Some(target) = self.targets.get(index) else {
            return Ok(());
        };
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.current = Some(std::io::BufWriter::new(std::fs::File::create(target)?));
        Ok(())
    }

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        if let Some(writer) = self.current.as_mut() {
            writer.write_all(bytes)?;
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), ExtensionError> {
        if let Some(mut writer) = self.current.take() {
            writer.flush()?;
        }
        if let Some(name) = self.names.get(self.at) {
            self.produced.push(name.clone());
        }
        Ok(())
    }
}

const BATCH_BYTES: usize = 8 << 20;

pub struct ToZip {
    writer: Option<zip::Writer<std::io::BufWriter<std::fs::File>>>,
    names: Vec<String>,
    buffer: Vec<u8>,
    batch: Vec<(String, Vec<u8>)>,
    batch_bytes: usize,
    threads: usize,
    at: usize,
    compress: bool,
    entries: usize,
}

impl ToZip {
    pub fn new(dest: &Path, compress: bool) -> Result<Self, ExtensionError> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self {
            writer: Some(zip::Writer::new(std::io::BufWriter::new(
                std::fs::File::create(dest)?,
            ))),
            names: Vec::new(),
            buffer: Vec::new(),
            batch: Vec::new(),
            batch_bytes: 0,
            threads: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
            at: 0,
            compress,
            entries: 0,
        })
    }

    #[must_use]
    pub const fn entries(&self) -> usize {
        self.entries
    }

    fn flush_batch(&mut self) -> Result<(), ExtensionError> {
        if self.batch.is_empty() {
            return Ok(());
        }
        let batch = std::mem::take(&mut self.batch);
        self.batch_bytes = 0;
        for prepared in crate::compress_batch(batch, self.compress, self.threads) {
            if let Some(writer) = self.writer.as_mut() {
                writer.add_prepared(prepared)?;
            }
        }
        Ok(())
    }

    pub fn close(mut self) -> Result<usize, ExtensionError> {
        self.close_in_place()?;
        Ok(self.entries)
    }

    fn close_in_place(&mut self) -> Result<(), ExtensionError> {
        self.flush_batch()?;
        if let Some(writer) = self.writer.take() {
            let mut out = writer.finish()?;
            out.flush()?;
        }
        Ok(())
    }
}

impl EntrySink for ToZip {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
        let root = Path::new("");
        self.names = validate(entries, root)?
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect();
        Ok(())
    }

    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
        self.at = index;
        self.buffer.clear();
        self.buffer.reserve(entry.size as usize);
        Ok(())
    }

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn end(&mut self) -> Result<(), ExtensionError> {
        let name = self.names.get(self.at).cloned().unwrap_or_default();
        let body = std::mem::take(&mut self.buffer);
        self.batch_bytes = self.batch_bytes.saturating_add(body.len());
        self.batch.push((name, body));
        self.entries += 1;

        if self.batch_bytes >= BATCH_BYTES {
            self.flush_batch()?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), ExtensionError> {
        self.close_in_place()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split::Splitter;

    pub(super) fn build(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(crate::MAGIC);
        out.push(3);
        out.extend_from_slice(&0_u64.to_le_bytes());
        out.extend_from_slice(&1_u64.to_le_bytes());
        out.push(0);
        out.extend_from_slice(b"Sink\0desc\0author\0");
        out.extend_from_slice(&1_i32.to_le_bytes());
        for (index, (path, body)) in files.iter().enumerate() {
            out.extend_from_slice(&(index as u32 + 1).to_le_bytes());
            out.extend_from_slice(path.as_bytes());
            out.push(0);
            out.extend_from_slice(&(body.len() as i64).to_le_bytes());
            out.extend_from_slice(&0_u32.to_le_bytes());
        }
        out.extend_from_slice(&0_u32.to_le_bytes());
        for (_, body) in files {
            out.extend_from_slice(body);
        }
        out
    }

    pub(super) struct Scratch(pub PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    pub(super) fn scratch(name: &str) -> Scratch {
        let base = std::env::var("TAPLINE_TEST_DIR").unwrap_or_else(|_| {
            format!(
                "{}/.cache/tapline-test",
                std::env::var("HOME").unwrap_or_else(|_| ".".into())
            )
        });
        let path = PathBuf::from(base).join(format!("sink-{name}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("mkdir");
        Scratch(path)
    }

    #[test]
    fn a_directory_sink_writes_the_files() {
        let raw = build(&[("lua/a.lua", b"print(1)"), ("b.txt", b"two")]);
        let dir = scratch("dir");
        let mut splitter = Splitter::new(ToDirectory::new(&dir.0));
        for piece in raw.chunks(5) {
            splitter.push(piece).expect("push");
        }
        let produced = splitter.finish().expect("finish").into_produced();

        assert_eq!(produced, vec!["lua/a.lua", "b.txt"]);
        assert_eq!(
            std::fs::read(dir.0.join("lua/a.lua")).expect("read"),
            b"print(1)"
        );
    }

    #[test]
    fn a_zip_sink_writes_an_archive_a_reader_accepts() {
        let raw = build(&[("a.txt", &vec![b'a'; 5000]), ("b.txt", b"short")]);
        let dir = scratch("zip");
        let zip_path = dir.0.join("out.zip");

        let mut splitter = Splitter::new(ToZip::new(&zip_path, true).expect("create"));
        for piece in raw.chunks(7) {
            splitter.push(piece).expect("push");
        }
        let count = splitter.finish().expect("finish").close().expect("close");
        assert_eq!(count, 2);

        let bytes = std::fs::read(&zip_path).expect("read");
        assert_eq!(bytes.get(..2), Some(&b"PK"[..]), "no PK signature");
        assert!(bytes.len() < 1000, "not compressed: {} bytes", bytes.len());

        if let Ok(output) = std::process::Command::new("unzip")
            .arg("-t")
            .arg(&zip_path)
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            assert!(output.status.success(), "unzip rejected it:\n{text}");
        }
    }

    #[test]
    fn a_streamed_zip_holds_one_entry_not_the_archive() {
        let raw = build(&[("big", &vec![7_u8; 100_000]), ("small", b"x")]);
        let dir = scratch("zipmem");
        let mut splitter = Splitter::new(ToZip::new(&dir.0.join("m.zip"), false).expect("create"));
        for piece in raw.chunks(4096) {
            splitter.push(piece).expect("push");
        }
        let sink = splitter.finish().expect("finish");
        assert_eq!(sink.entries(), 2);
        assert_eq!(
            sink.buffer.capacity(),
            0,
            "the last entry's buffer was kept"
        );
        assert!(sink.batch_bytes <= BATCH_BYTES, "batch grew past its bound");
        sink.close().expect("close");
    }

    #[test]
    fn an_escaping_path_is_refused_by_both_sinks() {
        let raw = build(&[("../../etc/passwd", b"no"), ("ok", b"yes")]);

        let dir = scratch("escape");
        let mut to_dir = Splitter::new(ToDirectory::new(&dir.0));
        let error = to_dir.push(&raw).expect_err("directory sink must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
        assert!(!dir.0.join("ok").exists(), "a file was written anyway");

        let mut to_zip = Splitter::new(ToZip::new(&dir.0.join("e.zip"), false).expect("create"));
        let error = to_zip.push(&raw).expect_err("zip sink must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
    }

    #[test]
    fn a_zip_without_close_is_not_silently_valid() {
        let dir = scratch("noclose");
        let path = dir.0.join("partial.zip");
        let raw = build(&[("a", b"body")]);
        let mut splitter = Splitter::new(ToZip::new(&path, false).expect("create"));
        splitter.push(&raw).expect("push");
        drop(splitter);

        let bytes = std::fs::read(&path).expect("read");
        let end = 0x0605_4b50_u32.to_le_bytes();
        assert!(
            !bytes.windows(4).any(|w| w == end),
            "an unclosed archive should have no end record"
        );
    }
}

pub struct Filtered<S: EntrySink> {
    inner: S,
    patterns: crate::glob::Patterns,
    passing: bool,
    passed: usize,
}

impl<S: EntrySink> Filtered<S> {
    pub const fn new(inner: S, patterns: crate::glob::Patterns) -> Self {
        Self {
            inner,
            patterns,
            passing: false,
            passed: 0,
        }
    }

    pub const fn passed(&self) -> usize {
        self.passed
    }

    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: EntrySink> EntrySink for Filtered<S> {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
        self.inner.index(entries)
    }

    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
        self.passing = self.patterns.selects(&entry.path);
        if self.passing {
            self.inner.begin(entry, index)?;
        }
        Ok(())
    }

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        if self.passing {
            self.inner.data(bytes)?;
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), ExtensionError> {
        if self.passing {
            self.inner.end()?;
            self.passed += 1;
        }
        self.passing = false;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), ExtensionError> {
        self.inner.finish()
    }
}

#[derive(Default)]
pub struct Fanout {
    sinks: Vec<Box<dyn EntrySink + Send>>,
}

impl Fanout {
    #[must_use]
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    #[must_use]
    pub fn with(mut self, sink: Box<dyn EntrySink + Send>) -> Self {
        self.sinks.push(sink);
        self
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sinks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sinks.is_empty()
    }
}

impl EntrySink for Fanout {
    fn index(&mut self, entries: &[ArchiveEntry]) -> Result<(), ExtensionError> {
        for sink in &mut self.sinks {
            sink.index(entries)?;
        }
        Ok(())
    }

    fn begin(&mut self, entry: &ArchiveEntry, index: usize) -> Result<(), ExtensionError> {
        for sink in &mut self.sinks {
            sink.begin(entry, index)?;
        }
        Ok(())
    }

    fn data(&mut self, bytes: &[u8]) -> Result<(), ExtensionError> {
        for sink in &mut self.sinks {
            sink.data(bytes)?;
        }
        Ok(())
    }

    fn end(&mut self) -> Result<(), ExtensionError> {
        for sink in &mut self.sinks {
            sink.end()?;
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), ExtensionError> {
        let mut first = Ok(());
        for sink in &mut self.sinks {
            let result = sink.finish();
            if first.is_ok() {
                first = result;
            }
        }
        first
    }
}

#[cfg(test)]
mod composition_tests {
    use super::*;
    use crate::glob::Patterns;
    use crate::split::Splitter;

    fn archive() -> Vec<u8> {
        super::tests::build(&[
            ("lua/a.lua", b"lua one"),
            ("materials/b.vmt", b"a material"),
            ("lua/deep/c.lua", b"lua two"),
        ])
    }

    #[test]
    fn a_filter_keeps_only_what_matches() {
        let dir = super::tests::scratch("filtered");
        let sink = Filtered::new(ToDirectory::new(&dir.0), Patterns::all().with("lua/**"));
        let mut splitter = Splitter::new(sink);
        for piece in archive().chunks(6) {
            splitter.push(piece).expect("push");
        }
        let produced = splitter
            .finish()
            .expect("finish")
            .into_inner()
            .into_produced();

        assert_eq!(produced, vec!["lua/a.lua", "lua/deep/c.lua"]);
        assert!(dir.0.join("lua/a.lua").exists());
        assert!(
            !dir.0.join("materials/b.vmt").exists(),
            "filtered file written"
        );
    }

    #[test]
    fn a_fanout_writes_every_destination_in_one_pass() {
        let dir = super::tests::scratch("fanout");
        let zip_path = dir.0.join("out.zip");
        let unpacked = dir.0.join("unpacked");

        let fanout = Fanout::new()
            .with(Box::new(ToDirectory::new(&unpacked)))
            .with(Box::new(ToZip::new(&zip_path, true).expect("zip")));
        assert_eq!(fanout.len(), 2);

        let mut splitter = Splitter::new(fanout);
        for piece in archive().chunks(5) {
            splitter.push(piece).expect("push");
        }
        splitter.finish().expect("finish");

        assert!(unpacked.join("lua/a.lua").exists(), "directory missing");
        assert!(zip_path.exists(), "zip missing");
    }

    #[test]
    fn filtering_and_fanning_out_compose() {
        let dir = super::tests::scratch("both");
        let unpacked = dir.0.join("unpacked");
        let sink = Filtered::new(
            Fanout::new().with(Box::new(ToDirectory::new(&unpacked))),
            Patterns::all().with("materials/**"),
        );
        let mut splitter = Splitter::new(sink);
        splitter.push(&archive()).expect("push");
        splitter.finish().expect("finish");

        assert!(unpacked.join("materials/b.vmt").exists());
        assert!(!unpacked.join("lua/a.lua").exists());
    }

    #[test]
    fn a_filter_that_matches_nothing_produces_an_empty_result() {
        let dir = super::tests::scratch("nomatch");
        let sink = Filtered::new(
            ToDirectory::new(&dir.0),
            Patterns::all().with("does/not/exist/**"),
        );
        let mut splitter = Splitter::new(sink);
        splitter.push(&archive()).expect("push");
        let produced = splitter
            .finish()
            .expect("finish")
            .into_inner()
            .into_produced();
        assert!(produced.is_empty());
    }

    #[test]
    fn a_boxed_zip_sink_still_writes_its_central_directory() {
        let dir = super::tests::scratch("boxedzip");
        let zip_path = dir.0.join("boxed.zip");
        let mut fanout = Fanout::new().with(Box::new(ToZip::new(&zip_path, true).expect("zip")));

        let mut splitter = Splitter::new(&mut fanout);
        splitter.push(&archive()).expect("push");
        splitter.finish().expect("finish");
        EntrySink::finish(&mut fanout).expect("finish must close the zip");

        let bytes = std::fs::read(&zip_path).expect("read");
        let end = 0x0605_4b50_u32.to_le_bytes();
        assert!(
            bytes.windows(4).any(|w| w == end),
            "no end-of-central-directory record"
        );
    }

    #[test]
    fn an_empty_fanout_consumes_the_stream_without_writing() {
        let mut splitter = Splitter::new(Fanout::new());
        splitter.push(&archive()).expect("push");
        let fanout = splitter.finish().expect("finish");
        assert!(fanout.is_empty());
    }

    #[test]
    fn a_filter_still_shows_every_path_to_the_sink_for_validation() {
        let raw = super::tests::build(&[("../escape", b"x"), ("lua/ok.lua", b"y")]);
        let dir = super::tests::scratch("filtervalidate");
        let sink = Filtered::new(ToDirectory::new(&dir.0), Patterns::all().with("lua/**"));
        let mut splitter = Splitter::new(sink);
        let error = splitter.push(&raw).expect_err("must refuse");
        assert!(
            matches!(error, ExtensionError::UnsafePath { .. }),
            "{error}"
        );
    }
}
