//! Writing content to disk with positional writes.

use std::fs::File;
use std::io;
use std::os::unix::fs::FileExt;
use std::path::Path;
use tapline_io::Sink;

/// A file open for positional writes.
pub struct FileSink {
    file: File,
}

impl FileSink {
    /// Creates the file and its parents; the path must be pre-validated by `tapline-fs`.
    pub fn create(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        Ok(Self { file })
    }

    /// Opens an existing file for update, keeping its contents.
    pub fn open_existing(path: &Path) -> io::Result<Self> {
        let file = File::options().write(true).read(true).open(path)?;
        Ok(Self { file })
    }

    /// Flushes to disk, blocking; put it on a blocking thread.
    pub fn sync_blocking(&self) -> io::Result<()> {
        self.file.sync_all()
    }

    /// Reads `len` bytes at `offset`, for verifying what is already on disk.
    pub fn read_at(&self, offset: u64, len: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0_u8; len];
        self.file.read_exact_at(&mut buffer, offset)?;
        Ok(buffer)
    }
}

impl Sink for FileSink {
    async fn write_at(&self, offset: u64, data: &[u8]) -> io::Result<()> {
        // Positional write: no shared offset, so concurrent callers cannot interleave.
        self.file.write_all_at(data, offset)
    }

    async fn allocate(&self, len: u64) -> io::Result<()> {
        self.file.set_len(len)
    }

    async fn sync(&self) -> io::Result<()> {
        self.file.sync_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_io::testing::block_on;

    /// Scratch dir, never under `/tmp`: that is tmpfs on the dev machine.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
                |_| {
                    let mut home = std::env::var("HOME").unwrap_or_else(|_| ".".to_owned());
                    home.push_str("/.cache/tapline-test");
                    std::path::PathBuf::from(home)
                },
                std::path::PathBuf::from,
            );
            let path = base.join(name);
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory must be creatable");
            Self(path)
        }

        fn join(&self, name: &str) -> std::path::PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn out_of_order_writes_assemble_the_same_file() {
        let scratch = Scratch::new("sink-order");
        let path = scratch.join("nested/dir/file.bin");

        let sink = FileSink::create(&path).expect("must create, including parents");
        block_on(async {
            sink.allocate(9).await.expect("allocate");
            sink.write_at(6, b"ghi").await.expect("tail");
            sink.write_at(0, b"abc").await.expect("head");
            sink.write_at(3, b"def").await.expect("middle");
            sink.sync().await.expect("sync");
        });

        assert_eq!(std::fs::read(&path).expect("read back"), b"abcdefghi");
    }

    #[test]
    fn allocation_sets_the_final_size_up_front() {
        let scratch = Scratch::new("sink-allocate");
        let path = scratch.join("sized.bin");

        let sink = FileSink::create(&path).expect("create");
        block_on(async { sink.allocate(4096).await.expect("allocate") });

        let metadata = std::fs::metadata(&path).expect("stat");
        assert_eq!(metadata.len(), 4096);
    }

    #[test]
    fn an_existing_file_can_be_read_back_for_verification() {
        let scratch = Scratch::new("sink-resume");
        let path = scratch.join("partial.bin");
        std::fs::write(&path, b"0123456789").expect("seed");

        let sink = FileSink::open_existing(&path).expect("open");
        assert_eq!(sink.read_at(3, 4).expect("read"), b"3456");

        block_on(async { sink.write_at(3, b"XXXX").await.expect("overwrite") });
        assert_eq!(std::fs::read(&path).expect("read back"), b"012XXXX789");
    }

    #[test]
    fn concurrent_writes_do_not_interleave() {
        let scratch = Scratch::new("sink-concurrent");
        let path = scratch.join("parallel.bin");

        let sink = std::sync::Arc::new(FileSink::create(&path).expect("create"));
        block_on(async { sink.allocate(4096).await.expect("allocate") });

        std::thread::scope(|scope| {
            for index in 0..8_u64 {
                let sink = std::sync::Arc::clone(&sink);
                scope.spawn(move || {
                    let payload = vec![b'a' + index as u8; 512];
                    block_on(async {
                        sink.write_at(index * 512, &payload).await.expect("write");
                    });
                });
            }
        });

        let contents = std::fs::read(&path).expect("read back");
        for index in 0..8_usize {
            let expected = vec![b'a' + index as u8; 512];
            let start = index * 512;
            assert_eq!(
                contents.get(start..start + 512),
                Some(expected.as_slice()),
                "block {index} was corrupted by a concurrent write"
            );
        }
    }
}
