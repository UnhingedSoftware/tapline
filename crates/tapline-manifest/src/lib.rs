//! Depot manifests: what a build is made of.
//!
//! A manifest names every file in a depot, every chunk of every file, and the
//! SHA-1 of each chunk's plaintext. That last part is what makes a download
//! verifiable: the chunk id *is* its content hash, so a chunk that does not
//! hash to the id the manifest named is not the chunk that was asked for,
//! whatever served it.
//!
//! # The container, measured
//!
//! Fetched live from `cache8-iad1.steamcontent.com` on 2026-08-26 for depot
//! 232257, served as `application/x-steam-manifest`:
//!
//! ```text
//! PK\x03\x04 ...                    a ZIP, one deflated entry named "z"
//!   └─ 0x71F617D0  u32 len  ContentManifestPayload    the files and chunks
//!      0x1F4812BE  u32 len  ContentManifestMetadata   depot id, sizes, flags
//!      0x1B81B817  u32 len  ContentManifestSignature
//!      0x32C415AB                                     end marker, no length
//! ```
//!
//! # Filenames are encrypted
//!
//! In that same real manifest every `filename` is a 90-byte base64 blob, because
//! `filenames_encrypted` is set. Decrypting them needs the depot key, which
//! Steam grants only for content the session is entitled to. A manifest can
//! therefore be *parsed* without a key and cannot be *used* without one, and
//! this crate keeps that distinction: [`Manifest::parse`] never needs a key, and
//! [`Manifest::decrypt_filenames`] is a separate step that does.

mod container;
mod file;

pub use container::{MAX_MANIFEST, ManifestError, RawManifest};
pub use file::{Chunk, FileEntry, FileFlags, Manifest};
