//! Steam's protobuf messages.
//!
//! Every type here is generated from Valve's own schema by `cargo xtask
//! gen-proto` and committed to the repository. That is deliberate: it means
//! building tapline needs no `protoc`, no build script, and no protobuf library,
//! and it means a change to Valve's schema arrives as a reviewable diff rather
//! than as different code compiling on a different machine.
//!
//! The vendored `.proto` files and their provenance are in `protos/README.md`.
//!
//! # Reading the generated code
//!
//! * Enums are newtypes over `i32` with associated constants, not Rust enums, so
//!   a value Valve added after this was generated round-trips unchanged instead
//!   of failing to decode.
//! * `optional` and `required` both become `Option<T>`. proto2's `required` is a
//!   decode failure waiting to happen — a peer that omits the field makes the
//!   whole message unreadable — and a missing field is better reported as `None`.
//! * Names keep Valve's spelling, so generated code can be cross-referenced
//!   against the schema without a translation step.

// The generated modules are the crate's entire content, and they are documented
// as a set here rather than one `//!` line at a time by the generator.
#![allow(missing_docs)]

mod generated;

pub use generated::*;
