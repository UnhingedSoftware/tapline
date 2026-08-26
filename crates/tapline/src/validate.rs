//! Checking an install against its manifest, and repairing what is wrong.
//!
//! An update trusts the install record: same manifest id means nothing to do.
//! That is right almost always and wrong exactly when it matters — a half-
//! finished download, a disk that dropped a sector, a file someone edited by
//! hand. `validate` answers the question directly instead, by reading every
//! chunk back off disk and hashing it.
//!
//! The chunk id *is* the SHA-1 of its plaintext, so this needs no reference
//! copy and no checksum file: the manifest already says what every chunk should
//! hash to. A chunk that does not is refetched, and only that chunk — a single
//! bad megabyte in a 30 GB install costs one megabyte to repair.

use std::collections::BTreeMap;
use std::path::Path;
use tapline_manifest::Manifest;

/// What a validation found wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Damage {
    /// The file is not there at all.
    Missing,
    /// The file is the wrong length.
    ///
    /// Checked before hashing because it is free and localises the problem
    /// immediately — a truncated download looks exactly like this.
    WrongSize {
        /// What the manifest says.
        expected: u64,
        /// What is on disk.
        actual: u64,
    },
    /// Some of the file's chunks do not hash to what the manifest named.
    ///
    /// Carries the offsets, so repair refetches those chunks rather than the
    /// whole file.
    CorruptChunks(Vec<u64>),
    /// The file could not be read.
    Unreadable(String),
}

/// The outcome of checking an install.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    /// How many files were checked.
    pub files_checked: u64,
    /// How many bytes were read and hashed.
    pub bytes_checked: u64,
    /// What was wrong, by path.
    pub damaged: BTreeMap<String, Damage>,
}

impl ValidationReport {
    /// Whether the install is intact.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.damaged.is_empty()
    }

    /// How many chunks need refetching.
    #[must_use]
    pub fn chunks_to_repair(&self) -> usize {
        self.damaged
            .values()
            .map(|damage| match damage {
                Damage::CorruptChunks(offsets) => offsets.len(),
                // A missing or wrong-length file needs all of its chunks; the
                // caller knows how many from the manifest.
                _ => 1,
            })
            .sum()
    }
}

/// Checks every file in a manifest against what is on disk.
///
/// `read_chunk` is supplied by the caller so this stays testable without a
/// filesystem — it is handed a path relative to the install root, an offset and
/// a length, and returns the bytes it found.
pub fn validate_manifest<F>(manifest: &Manifest, root: &Path, mut read_chunk: F) -> ValidationReport
where
    F: FnMut(&Path, u64, usize) -> std::io::Result<Vec<u8>>,
{
    let mut report = ValidationReport::default();

    for file in manifest.regular_files() {
        report.files_checked += 1;

        let Ok(safe) = tapline_fs::validate_path(&file.path) else {
            // A path the manifest names that we would refuse to create is not
            // damage to repair; it is a manifest to stop trusting. Reported so
            // it is not silent.
            report.damaged.insert(
                file.path.clone(),
                Damage::Unreadable("the manifest names an unsafe path".to_owned()),
            );
            continue;
        };
        let target = safe.resolve(root);

        let metadata = match std::fs::metadata(&target) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                report.damaged.insert(file.path.clone(), Damage::Missing);
                continue;
            }
            Err(error) => {
                report
                    .damaged
                    .insert(file.path.clone(), Damage::Unreadable(error.to_string()));
                continue;
            }
        };

        // Free, and it catches a truncated download before any hashing.
        if metadata.len() != file.size {
            report.damaged.insert(
                file.path.clone(),
                Damage::WrongSize {
                    expected: file.size,
                    actual: metadata.len(),
                },
            );
            continue;
        }

        let mut corrupt = Vec::new();
        for chunk in &file.chunks {
            let bytes = match read_chunk(&target, chunk.offset, chunk.uncompressed_size as usize) {
                Ok(bytes) => bytes,
                Err(error) => {
                    report
                        .damaged
                        .insert(file.path.clone(), Damage::Unreadable(error.to_string()));
                    corrupt.clear();
                    break;
                }
            };
            report.bytes_checked += bytes.len() as u64;

            // The manifest already says what this should hash to. No reference
            // copy, no checksum file.
            if tapline_crypto::sha1(&bytes) != chunk.id {
                corrupt.push(chunk.offset);
            }
        }

        if !corrupt.is_empty() {
            report
                .damaged
                .insert(file.path.clone(), Damage::CorruptChunks(corrupt));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_ids::{DepotId, ManifestId};
    use tapline_manifest::{Chunk, FileEntry, FileFlags};

    /// Builds a chunk whose id is the real SHA-1 of `content`.
    fn chunk_for(content: &[u8], offset: u64) -> Chunk {
        Chunk {
            id: tapline_crypto::sha1(content),
            crc: 0,
            offset,
            uncompressed_size: content.len() as u32,
            compressed_size: content.len() as u32,
        }
    }

    fn manifest_with(files: Vec<FileEntry>) -> Manifest {
        Manifest {
            depot: DepotId(1),
            id: ManifestId(1),
            created: 0,
            total_size: files.iter().map(|f| f.size).sum(),
            unique_chunks: 0,
            files,
        }
    }

    fn file_with(path: &str, chunks: Vec<Chunk>) -> FileEntry {
        FileEntry {
            path: path.to_owned(),
            size: chunks.iter().map(|c| u64::from(c.uncompressed_size)).sum(),
            flags: FileFlags::default(),
            raw_flags: 0,
            link_target: None,
            chunks,
        }
    }

    /// A scratch directory that removes itself.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
                |_| {
                    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                        .join(".cache/tapline-test")
                },
                std::path::PathBuf::from,
            );
            let path = base.join(name);
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("scratch");
            Self(path)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Reads from a real file, which is what the runtime does.
    fn real_read(path: &Path, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        use std::os::unix::fs::FileExt;
        let file = std::fs::File::open(path)?;
        let mut buffer = vec![0_u8; len];
        file.read_exact_at(&mut buffer, offset)?;
        Ok(buffer)
    }

    #[test]
    fn an_intact_install_validates_clean() {
        let scratch = Scratch::new("validate-clean");
        let content = b"the quick brown fox jumps over the lazy dog";
        std::fs::write(scratch.0.join("a.txt"), content).expect("write");

        let manifest = manifest_with(vec![file_with("a.txt", vec![chunk_for(content, 0)])]);
        let report = validate_manifest(&manifest, &scratch.0, real_read);

        assert!(
            report.is_clean(),
            "an intact file was reported as damaged: {report:?}"
        );
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.bytes_checked, content.len() as u64);
    }

    #[test]
    fn a_missing_file_is_reported_as_missing() {
        let scratch = Scratch::new("validate-missing");
        let manifest = manifest_with(vec![file_with("gone.txt", vec![chunk_for(b"x", 0)])]);

        let report = validate_manifest(&manifest, &scratch.0, real_read);
        assert_eq!(report.damaged.get("gone.txt"), Some(&Damage::Missing));
    }

    #[test]
    fn a_truncated_file_is_caught_by_its_size_before_any_hashing() {
        // Free, and it localises a half-finished download immediately.
        let scratch = Scratch::new("validate-short");
        std::fs::write(scratch.0.join("a.txt"), b"short").expect("write");

        let manifest = manifest_with(vec![file_with(
            "a.txt",
            vec![chunk_for(b"a much longer expected content", 0)],
        )]);
        let report = validate_manifest(&manifest, &scratch.0, real_read);

        assert!(matches!(
            report.damaged.get("a.txt"),
            Some(Damage::WrongSize { .. })
        ));
        assert_eq!(
            report.bytes_checked, 0,
            "a wrong-size file was hashed anyway"
        );
    }

    #[test]
    fn a_corrupted_chunk_is_located_rather_than_condemning_the_file() {
        // The point of chunk-level validation: a single bad megabyte in a 30 GB
        // install costs one megabyte to repair.
        let scratch = Scratch::new("validate-corrupt");
        let first = b"first chunk contents!!!";
        let second = b"second chunk contents!!";
        assert_eq!(first.len(), second.len());

        let mut content = Vec::new();
        content.extend_from_slice(first);
        content.extend_from_slice(second);
        let path = scratch.0.join("a.bin");
        std::fs::write(&path, &content).expect("write");

        let manifest = manifest_with(vec![file_with(
            "a.bin",
            vec![chunk_for(first, 0), chunk_for(second, first.len() as u64)],
        )]);
        assert!(validate_manifest(&manifest, &scratch.0, real_read).is_clean());

        // Damage only the second chunk.
        let mut damaged = content.clone();
        if let Some(byte) = damaged.get_mut(first.len() + 3) {
            *byte ^= 0xFF;
        }
        std::fs::write(&path, &damaged).expect("write");

        let report = validate_manifest(&manifest, &scratch.0, real_read);
        assert_eq!(
            report.damaged.get("a.bin"),
            Some(&Damage::CorruptChunks(vec![first.len() as u64])),
            "the wrong chunk was blamed"
        );
        assert_eq!(report.chunks_to_repair(), 1);
    }

    #[test]
    fn an_unsafe_path_is_reported_rather_than_silently_skipped() {
        // Not damage to repair — a manifest to stop trusting. But never silent.
        let scratch = Scratch::new("validate-unsafe");
        let manifest = manifest_with(vec![file_with("../escape", vec![chunk_for(b"x", 0)])]);

        let report = validate_manifest(&manifest, &scratch.0, real_read);
        assert!(matches!(
            report.damaged.get("../escape"),
            Some(Damage::Unreadable(_))
        ));
    }

    #[test]
    fn an_empty_manifest_validates_clean() {
        let scratch = Scratch::new("validate-empty");
        let report = validate_manifest(&manifest_with(Vec::new()), &scratch.0, real_read);
        assert!(report.is_clean());
        assert_eq!(report.files_checked, 0);
    }
}
