//! Depot manifests: what a build is made of.

mod container;
mod file;

pub use container::{MAX_MANIFEST, ManifestError, RawManifest};
pub use file::{Chunk, FileEntry, FileFlags, Manifest};
