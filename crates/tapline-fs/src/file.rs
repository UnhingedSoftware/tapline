use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(unix)]
pub fn read_exact_at(file: &File, buffer: &mut [u8], offset: u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;

    file.read_exact_at(buffer, offset)
}

#[cfg(windows)]
pub fn read_exact_at(file: &File, buffer: &mut [u8], offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;

    let wanted = buffer.len();
    let mut filled = 0_usize;
    while filled < wanted {
        let Some(rest) = buffer.get_mut(filled..) else {
            break;
        };
        match file.seek_read(rest, offset.saturating_add(filled as u64)) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "the file ended before the requested range was filled",
                ));
            }
            Ok(count) => filled = filled.saturating_add(count),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn write_all_at(file: &File, data: &[u8], offset: u64) -> io::Result<()> {
    use std::os::unix::fs::FileExt;

    file.write_all_at(data, offset)
}

#[cfg(windows)]
pub fn write_all_at(file: &File, data: &[u8], offset: u64) -> io::Result<()> {
    use std::os::windows::fs::FileExt;

    let total = data.len();
    let mut written = 0_usize;
    while written < total {
        let Some(rest) = data.get(written..) else {
            break;
        };
        match file.seek_write(rest, offset.saturating_add(written as u64)) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "the write made no progress",
                ));
            }
            Ok(count) => written = written.saturating_add(count),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    let points_at_a_directory = link
        .parent()
        .map_or_else(|| target.to_path_buf(), |parent| parent.join(target))
        .is_dir();

    if points_at_a_directory {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

pub fn remove_existing(path: &Path) -> io::Result<()> {
    let file_type = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    let removed = if names_a_directory(&file_type) {
        std::fs::remove_dir(path)
    } else {
        std::fs::remove_file(path)
    };
    match removed {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        outcome => outcome,
    }
}

#[cfg(unix)]
fn names_a_directory(file_type: &std::fs::FileType) -> bool {
    file_type.is_dir()
}

#[cfg(windows)]
fn names_a_directory(file_type: &std::fs::FileType) -> bool {
    use std::os::windows::fs::FileTypeExt;

    file_type.is_dir() || file_type.is_symlink_dir()
}

#[cfg(unix)]
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
}

#[cfg(windows)]
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(mode & 0o200 == 0);
    std::fs::set_permissions(path, permissions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
                |_| {
                    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                        .join(".cache/tapline-test")
                },
                PathBuf::from,
            );
            let path = base.join(name);
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("the scratch directory must be creatable");
            Self(path)
        }

        fn join(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_positional_write_lands_at_the_offset_it_names() {
        let scratch = Scratch::new("file-write-at");
        let path = scratch.join("out.bin");
        let file = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .read(true)
            .open(&path)
            .expect("create");
        file.set_len(9).expect("allocate");

        write_all_at(&file, b"ghi", 6).expect("tail");
        write_all_at(&file, b"abc", 0).expect("head");
        write_all_at(&file, b"def", 3).expect("middle");

        assert_eq!(std::fs::read(&path).expect("read back"), b"abcdefghi");
    }

    #[test]
    fn a_positional_read_returns_the_range_it_names() {
        let scratch = Scratch::new("file-read-at");
        let path = scratch.join("in.bin");
        std::fs::write(&path, b"0123456789").expect("seed");

        let file = File::open(&path).expect("open");
        let mut buffer = [0_u8; 4];
        read_exact_at(&file, &mut buffer, 3).expect("read");

        assert_eq!(&buffer, b"3456");
    }

    #[test]
    fn a_read_running_past_the_end_is_an_error_not_a_short_buffer() {
        let scratch = Scratch::new("file-read-short");
        let path = scratch.join("small.bin");
        std::fs::write(&path, b"abc").expect("seed");

        let file = File::open(&path).expect("open");
        let mut buffer = [0_u8; 8];
        let error = read_exact_at(&file, &mut buffer, 0).expect_err("the range is not there");

        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn an_empty_read_is_satisfied_without_touching_the_file() {
        let scratch = Scratch::new("file-read-empty");
        let path = scratch.join("empty.bin");
        std::fs::write(&path, b"").expect("seed");

        let file = File::open(&path).expect("open");
        read_exact_at(&file, &mut [], 0).expect("nothing was asked for");
    }

    #[test]
    fn a_symlink_resolves_to_what_it_points_at() {
        let scratch = Scratch::new("file-symlink");
        let target = scratch.join("real.txt");
        let link = scratch.join("link.txt");
        std::fs::write(&target, b"contents").expect("seed");

        symlink(Path::new("real.txt"), &link).expect("symlink");

        assert_eq!(std::fs::read(&link).expect("read through"), b"contents");
    }

    #[test]
    fn removing_a_symlink_takes_the_link_and_leaves_its_target() {
        let scratch = Scratch::new("file-remove-link");
        let target = scratch.join("real");
        std::fs::create_dir_all(&target).expect("seed");
        std::fs::write(target.join("inside.txt"), b"kept").expect("seed");
        let link = scratch.join("link");
        symlink(Path::new("real"), &link).expect("symlink");

        remove_existing(&link).expect("remove");

        assert!(!link.exists(), "the link is still there");
        assert!(
            target.join("inside.txt").is_file(),
            "the target was removed along with the link"
        );
    }

    #[test]
    fn removing_a_path_that_is_not_there_is_not_an_error() {
        let scratch = Scratch::new("file-remove-missing");
        remove_existing(&scratch.join("never-existed")).expect("nothing to remove");
    }

    #[test]
    fn a_directory_with_something_in_it_refuses_to_be_removed() {
        let scratch = Scratch::new("file-remove-occupied");
        let occupied = scratch.join("occupied");
        std::fs::create_dir_all(&occupied).expect("seed");
        std::fs::write(occupied.join("inside.txt"), b"in the way").expect("seed");

        remove_existing(&occupied).expect_err("a non-empty directory is not ours to delete");
        assert!(
            occupied.join("inside.txt").is_file(),
            "the contents were removed anyway"
        );
    }

    #[test]
    fn an_empty_directory_gives_way_to_a_link() {
        let scratch = Scratch::new("file-remove-empty-dir");
        let target = scratch.join("real.txt");
        std::fs::write(&target, b"contents").expect("seed");
        let link = scratch.join("link.txt");
        std::fs::create_dir_all(&link).expect("seed");

        remove_existing(&link).expect("remove");
        symlink(Path::new("real.txt"), &link).expect("symlink");

        assert_eq!(std::fs::read(&link).expect("read through"), b"contents");
    }

    #[test]
    fn a_symlink_replaces_whatever_stood_in_its_place() {
        let scratch = Scratch::new("file-relink");
        let target = scratch.join("real.txt");
        std::fs::write(&target, b"contents").expect("seed");
        let link = scratch.join("link.txt");
        std::fs::write(&link, b"stale").expect("seed");

        remove_existing(&link).expect("remove");
        symlink(Path::new("real.txt"), &link).expect("symlink");

        assert_eq!(std::fs::read(&link).expect("read through"), b"contents");
    }

    #[test]
    fn clearing_the_write_bit_marks_the_file_read_only() {
        let scratch = Scratch::new("file-mode");
        let path = scratch.join("locked.txt");
        std::fs::write(&path, b"seed").expect("seed");

        set_mode(&path, 0o444).expect("set mode");
        assert!(
            std::fs::metadata(&path)
                .expect("stat")
                .permissions()
                .readonly(),
            "the file is still writable"
        );

        set_mode(&path, 0o644).expect("restore");
        assert!(
            !std::fs::metadata(&path)
                .expect("stat")
                .permissions()
                .readonly(),
            "the write bit did not come back"
        );
    }
}
