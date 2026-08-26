//! PICS: what an app is made of.
//!
//! Answers the question every install starts with — *which depots, at which
//! manifest ids?* — and it is the question with no HTTP equivalent. PICS is
//! CM-only, which is why an open session is the base capability of this whole
//! workspace rather than a detail of it.
//!
//! # The response is text KeyValues
//!
//! Measured against a live response for app 232250 on 2026-08-26: the `buffer`
//! field of `CMsgClientPICSProductInfoResponse` holds a NUL-terminated *text*
//! KeyValues document, the same format as an `appmanifest`. It is not the binary
//! KeyValues variant, and writing a binary parser for it — which is what a
//! reasonable person would have assumed — would have produced a parser with
//! nothing to parse.
//!
//! # The shape it has
//!
//! ```text
//! "appinfo"
//! {
//!     "appid"  "232250"
//!     "common" { "name" "Team Fortress 2 Dedicated Server"  "oslist" "windows,linux" }
//!     "depots"
//!     {
//!         "232250" { "manifests" { "public" { "gid" "3447236868550150350" ... } } }
//!         "232256" { "config" { "oslist" "linux" }  "manifests" { ... } }
//!         "branches" { "public" { "buildid" "..." } }
//!     }
//! }
//! ```
//!
//! Note that `depots` mixes real depot ids with the `branches` key and a few
//! other non-depot entries, so a reader that treats every child as a depot ends
//! up trying to download one called "branches".

mod app;
mod client;

pub use app::{AppInfo, Branch, Depot, DepotFilter, Os};
pub use client::{PicsError, product_info};
