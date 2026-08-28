//! Where a download is allowed to write.

mod path;

pub use path::{PathError, SafePath, validate_path, validate_symlink};
