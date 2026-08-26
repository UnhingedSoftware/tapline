//! Working out what an update actually has to fetch.
//!
//! A depot's content is chunked and every chunk is named by the SHA-1 of its
//! plaintext. Two builds of the same depot therefore share every chunk whose
//! content did not change, and a manifest diff is a set difference over chunk
//! ids. That is where a "20 GB update" becomes 200 MB.
//!
//! Two independent sources of reuse, and both matter:
//!
//! * **The old manifest.** If the depot is installed at a known manifest, the
//!   chunks that appear in both are already on disk at known offsets, and can be
//!   copied locally rather than downloaded.
//! * **What is actually on disk.** A resumed download has files that are
//!   partially correct, and a `validate` run has files that may have rotted.
//!   Reading a chunk back and hashing it answers the question directly, without
//!   trusting any record of what happened last time.
//!
//! The second is slower and always right; the first is instant and depends on
//! the install record being honest. An update uses the first, `validate` uses
//! the second, and a resume after a crash uses the second for the file it was
//! part-way through.

use std::collections::{HashMap, HashSet};
use tapline_manifest::{Chunk, Manifest};

/// Where a chunk's bytes will come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkSource {
    /// Fetch it from the CDN.
    Download,
    /// Copy it from somewhere already on disk.
    ///
    /// The path is relative to the install root, as the old manifest named it.
    Local {
        /// The file holding the bytes.
        path: String,
        /// Where in that file they start.
        offset: u64,
    },
}

/// What an update will do, chunk by chunk.
#[derive(Debug, Clone, Default)]
pub struct DeltaPlan {
    /// Chunks that must be fetched, with their stored size.
    pub download: Vec<Chunk>,
    /// Chunks available locally, and where from.
    pub local: Vec<(Chunk, ChunkSource)>,
    /// Bytes that must come over the network.
    pub download_bytes: u64,
    /// Bytes that will be reused from disk.
    pub reused_bytes: u64,
}

impl DeltaPlan {
    /// The fraction of the new build already present, from 0.0 to 1.0.
    #[must_use]
    pub fn reuse_ratio(&self) -> f64 {
        let total = self.download_bytes + self.reused_bytes;
        if total == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "a display ratio; f64 is exact below 8 PiB"
        )]
        {
            self.reused_bytes as f64 / total as f64
        }
    }
}

/// Diffs a new manifest against the build already installed.
///
/// `old` is the manifest the depot is currently at. Chunks present in both are
/// marked reusable, with the location the *old* manifest gave them — that is
/// where the bytes are on disk right now.
#[must_use]
pub fn diff(old: &Manifest, new: &Manifest) -> DeltaPlan {
    // Where each chunk currently lives. First occurrence wins: a chunk repeated
    // across files is the same bytes either way, and reading the first is as
    // good as reading the fifth.
    let mut existing: HashMap<[u8; 20], (String, u64)> = HashMap::new();
    for file in old.regular_files() {
        for chunk in &file.chunks {
            existing
                .entry(chunk.id)
                .or_insert_with(|| (file.path.clone(), chunk.offset));
        }
    }

    let mut plan = DeltaPlan::default();
    let mut seen: HashSet<[u8; 20]> = HashSet::new();

    for file in new.regular_files() {
        for chunk in &file.chunks {
            // Distinct by id: a chunk needed by three files is fetched once.
            if !seen.insert(chunk.id) {
                continue;
            }

            if let Some((path, offset)) = existing.get(&chunk.id) {
                plan.reused_bytes += u64::from(chunk.uncompressed_size);
                plan.local.push((
                    chunk.clone(),
                    ChunkSource::Local {
                        path: path.clone(),
                        offset: *offset,
                    },
                ));
            } else {
                plan.download_bytes += u64::from(chunk.compressed_size);
                plan.download.push(chunk.clone());
            }
        }
    }
    plan
}

/// What a full install would cost, with nothing to reuse.
#[must_use]
pub fn full(new: &Manifest) -> DeltaPlan {
    let mut plan = DeltaPlan::default();
    let (chunks, bytes) = new.distinct_chunks();
    plan.download_bytes = bytes;
    plan.download = chunks.into_iter().cloned().collect();
    plan
}

/// Files present in `old` but not in `new`.
///
/// An update must delete these. Leaving them behind is how an install
/// accumulates dead files across updates until it no longer matches what a
/// fresh install of the same build would produce — which is exactly what the
/// differential test against steamcmd would catch, and exactly what an operator
/// would not.
#[must_use]
pub fn removed_files(old: &Manifest, new: &Manifest) -> Vec<String> {
    let kept: HashSet<&str> = new.files.iter().map(|file| file.path.as_str()).collect();
    old.files
        .iter()
        .filter(|file| !kept.contains(file.path.as_str()))
        .map(|file| file.path.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_ids::{DepotId, ManifestId};
    use tapline_manifest::{FileEntry, FileFlags};

    fn chunk(id: u8, offset: u64, size: u32) -> Chunk {
        Chunk {
            id: [id; 20],
            crc: 0,
            offset,
            uncompressed_size: size,
            compressed_size: size / 2,
        }
    }

    fn manifest(id: u64, files: Vec<FileEntry>) -> Manifest {
        Manifest {
            depot: DepotId(1),
            id: ManifestId(id),
            created: 0,
            total_size: files.iter().map(|f| f.size).sum(),
            unique_chunks: 0,
            files,
        }
    }

    fn file(path: &str, chunks: Vec<Chunk>) -> FileEntry {
        FileEntry {
            path: path.to_owned(),
            size: chunks.iter().map(|c| u64::from(c.uncompressed_size)).sum(),
            flags: FileFlags::default(),
            raw_flags: 0,
            link_target: None,
            chunks,
        }
    }

    #[test]
    fn an_unchanged_build_downloads_nothing() {
        // The most important case for an operator: running an update when
        // nothing changed must not move a byte.
        let build = manifest(
            1,
            vec![file("a", vec![chunk(1, 0, 100), chunk(2, 100, 100)])],
        );
        let plan = diff(&build, &build);

        assert!(
            plan.download.is_empty(),
            "an identical build wanted a download"
        );
        assert_eq!(plan.download_bytes, 0);
        assert_eq!(plan.local.len(), 2);
        assert!((plan.reuse_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn only_the_changed_chunks_are_fetched() {
        // The claim the whole crate exists to support.
        let old = manifest(
            1,
            vec![file(
                "a",
                vec![chunk(1, 0, 100), chunk(2, 100, 100), chunk(3, 200, 100)],
            )],
        );
        let new = manifest(
            2,
            vec![file(
                "a",
                vec![chunk(1, 0, 100), chunk(9, 100, 100), chunk(3, 200, 100)],
            )],
        );

        let plan = diff(&old, &new);
        assert_eq!(
            plan.download.len(),
            1,
            "more than the changed chunk was fetched"
        );
        assert_eq!(plan.download.first().map(|c| c.id), Some([9; 20]));
        assert_eq!(plan.local.len(), 2);
    }

    #[test]
    fn a_chunk_that_moved_is_still_reused() {
        // Content addressing means a chunk that changed position is the same
        // bytes. Fetching it again would be paying for a memmove.
        let old = manifest(
            1,
            vec![file("a", vec![chunk(1, 0, 100), chunk(2, 100, 100)])],
        );
        let new = manifest(
            2,
            vec![file("a", vec![chunk(2, 0, 100), chunk(1, 100, 100)])],
        );

        let plan = diff(&old, &new);
        assert!(plan.download.is_empty(), "a reordered chunk was refetched");
        assert_eq!(plan.local.len(), 2);
    }

    #[test]
    fn a_chunk_that_moved_between_files_is_still_reused() {
        // And the source it is copied from is the file that actually holds it.
        let old = manifest(1, vec![file("a", vec![chunk(1, 0, 100)])]);
        let new = manifest(2, vec![file("b", vec![chunk(1, 0, 100)])]);

        let plan = diff(&old, &new);
        assert!(plan.download.is_empty());
        assert_eq!(
            plan.local.first().map(|(_, source)| source.clone()),
            Some(ChunkSource::Local {
                path: "a".to_owned(),
                offset: 0
            })
        );
    }

    #[test]
    fn a_repeated_chunk_is_counted_once() {
        let new = manifest(
            2,
            vec![
                file("a", vec![chunk(7, 0, 100)]),
                file("b", vec![chunk(7, 0, 100)]),
            ],
        );
        let plan = full(&new);
        assert_eq!(plan.download.len(), 1, "a shared chunk was fetched twice");
    }

    #[test]
    fn a_first_install_reuses_nothing() {
        let new = manifest(
            1,
            vec![file("a", vec![chunk(1, 0, 100), chunk(2, 100, 100)])],
        );
        let plan = full(&new);

        assert_eq!(plan.download.len(), 2);
        assert_eq!(plan.reused_bytes, 0);
        assert_eq!(plan.reuse_ratio(), 0.0);
    }

    #[test]
    fn a_realistic_update_reuses_almost_everything() {
        // The shape of a real patch: one changed chunk in two hundred.
        let old_chunks: Vec<Chunk> = (0..200).map(|i| chunk(i as u8, i * 1000, 1000)).collect();
        let mut new_chunks = old_chunks.clone();
        if let Some(slot) = new_chunks.get_mut(100) {
            slot.id = [201; 20];
        }

        let old = manifest(1, vec![file("big", old_chunks)]);
        let new = manifest(2, vec![file("big", new_chunks)]);

        let plan = diff(&old, &new);
        assert_eq!(plan.download.len(), 1);
        assert!(
            plan.reuse_ratio() > 0.99,
            "reuse ratio was only {}",
            plan.reuse_ratio()
        );
    }

    #[test]
    fn files_dropped_from_a_build_are_reported_for_deletion() {
        // Left behind, they accumulate across updates until the install no
        // longer matches what a fresh install of the same build would produce.
        let old = manifest(
            1,
            vec![
                file("keep", vec![chunk(1, 0, 10)]),
                file("gone", vec![chunk(2, 0, 10)]),
            ],
        );
        let new = manifest(2, vec![file("keep", vec![chunk(1, 0, 10)])]);

        assert_eq!(removed_files(&old, &new), vec!["gone".to_owned()]);
        assert!(removed_files(&new, &new).is_empty());
    }

    #[test]
    fn an_empty_plan_does_not_divide_by_zero() {
        assert_eq!(DeltaPlan::default().reuse_ratio(), 0.0);
    }
}
