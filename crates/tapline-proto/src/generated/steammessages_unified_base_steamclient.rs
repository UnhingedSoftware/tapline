//! Generated from `steammessages_unified_base.steamclient.proto`. Do not edit — run `cargo xtask gen-proto`.
//!
//! Provenance and regeneration are documented in
//! `crates/tapline-proto/protos/README.md`.
//!
//! Valve's own spelling is preserved throughout — `CPublishedFile_Vote_Request`
//! stays as written — so this can be cross-referenced against the schema
//! without a translation step. That is what the naming allows below are for.
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]
#![allow(unused_imports, clippy::doc_markdown, clippy::too_many_lines)]
#![allow(clippy::match_single_binding, clippy::struct_excessive_bools)]
#![allow(clippy::used_underscore_binding, clippy::unreadable_literal)]

use tapline_wire::{Decoder, Encoder, Message, WireError, WireType};

/// `EProtoExecutionSite`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EProtoExecutionSite(pub i32);

impl EProtoExecutionSite {
    /// `k_EProtoExecutionSiteUnknown` = `0`
    pub const k_EProtoExecutionSiteUnknown: Self = Self(0);
    /// `k_EProtoExecutionSiteSteamClient` = `2`
    pub const k_EProtoExecutionSiteSteamClient: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EProtoExecutionSite {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EProtoServiceType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EProtoServiceType(pub i32);

impl EProtoServiceType {
    /// `k_EProtoServiceTypeSteamMessages` = `0`
    pub const k_EProtoServiceTypeSteamMessages: Self = Self(0);
    /// `k_EProtoServiceTypeVRGamepadUIMessages` = `1`
    pub const k_EProtoServiceTypeVRGamepadUIMessages: Self = Self(1);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EProtoServiceType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `NoResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NoResponse {}

impl Message for NoResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, _encoder: &mut Encoder) {}
}
