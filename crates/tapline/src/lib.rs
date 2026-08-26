//! Install Steam apps.
//!
//! The crate other projects link. Everything below it — the CM session, PICS,
//! the manifest format, the chunk pipeline — is assembled here into the two
//! operations a caller actually wants:
//!
//! ```no_run
//! # async fn example() -> Result<(), tapline::InstallError> {
//! use tapline::{Session, InstallOptions};
//! use tapline_ids::AppId;
//!
//! let mut session = Session::anonymous().await?;
//!
//! // What would it cost? No bytes are fetched to answer this.
//! let plan = session.plan(AppId(232_250), &InstallOptions::default()).await?;
//! println!("{} to download, {} reused", plan.download_bytes, plan.reused_bytes);
//!
//! // Do it.
//! session.install(AppId(232_250), &InstallOptions::default()).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Why `plan` is separate
//!
//! A scheduler placing an install on a node wants the byte cost before it
//! commits the disk, and an operator wants to know that a "20 GB update" is
//! really 200 MB of changed chunks. steamcmd cannot answer either question
//! without doing the download.

mod delta;
mod install;
mod session;
mod validate;
mod workshop;

pub use delta::{ChunkSource, DeltaPlan, diff, full, removed_files};
pub use install::{InstallError, InstallOptions, InstallReport};
pub use session::Session;
pub use validate::{Damage, ValidationReport, validate_manifest};
pub use workshop::{WorkshopContent, WorkshopError, WorkshopItem, classify, item_dir, options_for};

pub use tapline_event::{Event, Plan, RetryReason};
pub use tapline_ids::{AppId, DepotId, ManifestId, PublishedFileId};
pub use tapline_pics::Os;
