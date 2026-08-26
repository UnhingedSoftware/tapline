//! Where a download is allowed to write.
//!
//! This is the sharpest edge in the project. A manifest's filenames are
//! attacker-influenced for Workshop content — anyone can publish a Workshop item,
//! and its manifest names the paths tapline will create. A downloader that
//! joined those names onto an install directory and started writing would let a
//! published item drop a file anywhere the process can reach.
//!
//! So paths are not joined; they are **validated first and rejected on any
//! doubt**. The rules, each of which exists because skipping it is exploitable:
//!
//! * **No absolute paths.** `/etc/cron.d/x` must not become `/etc/cron.d/x`.
//! * **No parent traversal.** Not just a leading `../`, but any `..` component
//!   anywhere — `a/../../b` normalises out of the root just as well.
//! * **No root or prefix components.** On Windows a path may carry a drive
//!   letter or a UNC prefix, and `C:\Windows\...` inside a manifest is the same
//!   attack in different clothing.
//! * **No empty or `.`-only paths**, which resolve to the root itself and would
//!   turn a file write into a directory clobber.
//! * **Symlink targets are validated the same way**, and additionally must not
//!   escape when resolved against the link's own directory. A symlink is the
//!   indirect version of the same attack: `link -> ../../etc`, then write
//!   through it.
//! * **No NUL bytes**, which truncate a path at the syscall boundary and make
//!   what was validated differ from what is opened.
//!
//! What this crate deliberately does *not* do is decide these questions by
//! calling `canonicalize` on the filesystem. That answers a question about the
//! disk as it is at that instant, and the answer can change before the file is
//! opened. Validation here is a pure function of the path text; the runtime adds
//! `O_NOFOLLOW` on top when it opens.

mod path;

pub use path::{PathError, SafePath, validate_path, validate_symlink};
