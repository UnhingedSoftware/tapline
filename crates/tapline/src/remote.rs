//! Reading part of a file that has not been downloaded.

use crate::install::InstallError;
use std::sync::Arc;
use tapline_ids::DepotId;
use tapline_manifest::Chunk;

/// A file in a depot, readable without downloading all of it.
pub struct RemoteFile {
    chunks: Vec<Chunk>,
    size: u64,
    depot: DepotId,
    key: [u8; 32],
    hosts: Vec<String>,
    http: Arc<tapline_rt_tokio::HttpClient>,
    limit: Arc<tokio::sync::Semaphore>,
}

impl std::fmt::Debug for RemoteFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteFile")
            .field("size", &self.size)
            .field("chunks", &self.chunks.len())
            .field("depot", &self.depot)
            .finish_non_exhaustive()
    }
}

impl RemoteFile {
    pub(crate) fn new(
        mut chunks: Vec<Chunk>,
        depot: DepotId,
        key: [u8; 32],
        hosts: Vec<String>,
        http: Arc<tapline_rt_tokio::HttpClient>,
        limit: Arc<tokio::sync::Semaphore>,
    ) -> Self {
        // The manifest does not promise offset order; every lookup below assumes it.
        chunks.sort_by_key(|chunk| chunk.offset);
        let size = chunks
            .last()
            .map_or(0, |chunk| chunk.offset + u64::from(chunk.uncompressed_size));
        Self {
            chunks,
            size,
            depot,
            key,
            hosts,
            http,
            limit,
        }
    }

    /// The file's size.
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.size
    }

    /// Whether the file is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// How many chunks the file is stored as.
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Which chunks a byte range crosses.
    #[must_use]
    pub fn chunks_for(&self, offset: u64, len: u64) -> Vec<usize> {
        if len == 0 {
            return Vec::new();
        }
        let end = offset.saturating_add(len);
        self.chunks
            .iter()
            .enumerate()
            .filter(|(_, chunk)| {
                let chunk_end = chunk.offset + u64::from(chunk.uncompressed_size);
                chunk.offset < end && chunk_end > offset
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// How many bytes fetching these ranges would transfer.
    #[must_use]
    pub fn cost_of(&self, ranges: &[(u64, u64)]) -> u64 {
        let mut wanted = std::collections::BTreeSet::new();
        for (offset, len) in ranges {
            wanted.extend(self.chunks_for(*offset, *len));
        }
        wanted
            .iter()
            .filter_map(|index| self.chunks.get(*index))
            .map(|chunk| u64::from(chunk.compressed_size))
            .sum()
    }

    /// Reads one range.
    pub async fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>, InstallError> {
        let mut parts = self.read_many(&[(offset, len)]).await?;
        Ok(parts.pop().unwrap_or_default())
    }

    /// Reads several ranges, fetching each chunk at most once.
    pub async fn read_many(&self, ranges: &[(u64, u64)]) -> Result<Vec<Vec<u8>>, InstallError> {
        let mut wanted: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for (offset, len) in ranges {
            wanted.extend(self.chunks_for(*offset, *len));
        }

        let mut fetched: std::collections::BTreeMap<usize, Vec<u8>> =
            std::collections::BTreeMap::new();
        let mut tasks: tokio::task::JoinSet<(usize, Result<Vec<u8>, InstallError>)> =
            tokio::task::JoinSet::new();

        for index in wanted {
            let Some(chunk) = self.chunks.get(index).cloned() else {
                continue;
            };
            let http = Arc::clone(&self.http);
            let hosts = self.hosts.clone();
            let limit = Arc::clone(&self.limit);
            let key = self.key;
            let depot = self.depot;

            tasks.spawn(async move {
                let outcome = async move {
                    // Draws on the same process-wide budget as ordinary downloads.
                    let _permit = limit
                        .acquire_owned()
                        .await
                        .map_err(|error| InstallError::Io(error.to_string()))?;
                    crate::session::fetch_and_decode(&http, &hosts, depot, &chunk, &key, index)
                        .await
                }
                .await;
                (index, outcome)
            });
        }

        while let Some(joined) = tasks.join_next().await {
            let (index, outcome) = joined
                .map_err(|error| InstallError::Io(format!("a ranged read failed: {error}")))?;
            fetched.insert(index, outcome?);
        }

        let mut out = Vec::with_capacity(ranges.len());
        for (offset, len) in ranges {
            let mut piece = Vec::with_capacity(*len as usize);
            for index in self.chunks_for(*offset, *len) {
                let (Some(chunk), Some(bytes)) = (self.chunks.get(index), fetched.get(&index))
                else {
                    continue;
                };
                let start = offset.saturating_sub(chunk.offset) as usize;
                let stop = ((offset + len).saturating_sub(chunk.offset) as usize).min(bytes.len());
                if let Some(slice) = bytes.get(start..stop) {
                    piece.extend_from_slice(slice);
                }
            }
            out.push(piece);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(count: usize, size: u32) -> RemoteFile {
        let chunks = (0..count)
            .map(|index| Chunk {
                id: [0; 20],
                crc: 0,
                offset: (index as u64) * u64::from(size),
                uncompressed_size: size,
                // Half, so the cost figures are distinguishable from the sizes.
                compressed_size: size / 2,
            })
            .collect();
        RemoteFile::new(
            chunks,
            DepotId(4021),
            [0; 32],
            vec!["host.invalid".to_owned()],
            Arc::new(tapline_rt_tokio::HttpClient::new()),
            Arc::new(tokio::sync::Semaphore::new(1)),
        )
    }

    #[test]
    fn the_size_is_the_end_of_the_last_chunk() {
        let file = file(4, 1000);
        assert_eq!(file.len(), 4000);
        assert_eq!(file.chunk_count(), 4);
        assert!(!file.is_empty());
    }

    #[test]
    fn an_empty_file_reads_as_empty() {
        let file = file(0, 1000);
        assert_eq!(file.len(), 0);
        assert!(file.is_empty());
        assert!(file.chunks_for(0, 10).is_empty());
    }

    #[test]
    fn a_range_inside_one_chunk_needs_only_that_chunk() {
        let file = file(4, 1000);
        assert_eq!(file.chunks_for(10, 20), vec![0]);
        assert_eq!(file.chunks_for(1500, 100), vec![1]);
    }

    #[test]
    fn a_range_across_a_boundary_needs_both() {
        let file = file(4, 1000);
        assert_eq!(file.chunks_for(990, 20), vec![0, 1]);
        assert_eq!(file.chunks_for(500, 2000), vec![0, 1, 2]);
    }

    #[test]
    fn a_zero_length_range_needs_nothing() {
        let file = file(4, 1000);
        assert!(file.chunks_for(1500, 0).is_empty());
    }

    #[test]
    fn a_range_at_the_very_end_is_included() {
        let file = file(4, 1000);
        assert_eq!(file.chunks_for(3999, 1), vec![3]);
        assert_eq!(file.chunks_for(3000, 1000), vec![3]);
    }

    #[test]
    fn a_range_past_the_end_asks_for_no_more_than_exists() {
        let file = file(4, 1000);
        assert_eq!(file.chunks_for(3900, 500), vec![3]);
        assert!(file.chunks_for(5000, 10).is_empty());
    }

    #[test]
    fn cost_counts_each_chunk_once() {
        let file = file(4, 1000);
        assert_eq!(file.cost_of(&[(0, 10)]), 500);
        assert_eq!(file.cost_of(&[(0, 10), (20, 10)]), 500);
        assert_eq!(file.cost_of(&[(0, 10), (1500, 10)]), 1000);
    }

    #[test]
    fn cost_reports_what_a_selective_read_would_save() {
        let file = file(10, 1000);
        let everything = file.cost_of(&[(0, 10_000)]);
        let two_files = file.cost_of(&[(0, 100), (9000, 100)]);
        assert_eq!(everything, 5000);
        assert_eq!(two_files, 1000);
        assert!(two_files < everything, "a selective read should cost less");
    }

    #[test]
    fn chunks_are_sorted_even_when_the_manifest_is_not() {
        let chunks = vec![
            Chunk {
                id: [0; 20],
                crc: 0,
                offset: 1000,
                uncompressed_size: 1000,
                compressed_size: 500,
            },
            Chunk {
                id: [0; 20],
                crc: 0,
                offset: 0,
                uncompressed_size: 1000,
                compressed_size: 500,
            },
        ];
        let file = RemoteFile::new(
            chunks,
            DepotId(1),
            [0; 32],
            Vec::new(),
            Arc::new(tapline_rt_tokio::HttpClient::new()),
            Arc::new(tokio::sync::Semaphore::new(1)),
        );
        assert_eq!(file.len(), 2000);
        assert_eq!(file.chunks_for(0, 10), vec![0]);
        assert_eq!(file.chunks_for(1000, 10), vec![1]);
    }
}
