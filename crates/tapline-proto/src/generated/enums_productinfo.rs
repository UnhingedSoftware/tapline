//! Generated from `enums_productinfo.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EContentDescriptorID`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentDescriptorID(pub i32);

impl EContentDescriptorID {
    /// `k_EContentDescriptor_NudityOrSexualContent` = `1`
    pub const k_EContentDescriptor_NudityOrSexualContent: Self = Self(1);
    /// `k_EContentDescriptor_FrequentViolenceOrGore` = `2`
    pub const k_EContentDescriptor_FrequentViolenceOrGore: Self = Self(2);
    /// `k_EContentDescriptor_AdultOnlySexualContent` = `3`
    pub const k_EContentDescriptor_AdultOnlySexualContent: Self = Self(3);
    /// `k_EContentDescriptor_GratuitousSexualContent` = `4`
    pub const k_EContentDescriptor_GratuitousSexualContent: Self = Self(4);
    /// `k_EContentDescriptor_AnyMatureContent` = `5`
    pub const k_EContentDescriptor_AnyMatureContent: Self = Self(5);
    /// `k_EContentDescriptorMAX` = `6`
    pub const k_EContentDescriptorMAX: Self = Self(6);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentDescriptorID {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ERatingAgency`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ERatingAgency(pub i32);

impl ERatingAgency {
    /// `k_ERatingAgency_Invalid` = `0`
    pub const k_ERatingAgency_Invalid: Self = Self(0);
    /// `k_ERatingAgency_ESRB` = `1`
    pub const k_ERatingAgency_ESRB: Self = Self(1);
    /// `k_ERatingAgency_PEGI` = `2`
    pub const k_ERatingAgency_PEGI: Self = Self(2);
    /// `k_ERatingAgency_BBFC` = `3`
    pub const k_ERatingAgency_BBFC: Self = Self(3);
    /// `k_ERatingAgency_USK` = `4`
    pub const k_ERatingAgency_USK: Self = Self(4);
    /// `k_ERatingAgency_OFLC_AU` = `5`
    pub const k_ERatingAgency_OFLC_AU: Self = Self(5);
    /// `k_ERatingAgency_OFLC_NZ` = `6`
    pub const k_ERatingAgency_OFLC_NZ: Self = Self(6);
    /// `k_ERatingAgency_CERO` = `7`
    pub const k_ERatingAgency_CERO: Self = Self(7);
    /// `k_ERatingAgency_GRAC` = `8`
    pub const k_ERatingAgency_GRAC: Self = Self(8);
    /// `k_ERatingAgency_GMEDIA` = `9`
    pub const k_ERatingAgency_GMEDIA: Self = Self(9);
    /// `k_ERatingAgency_DEJUS` = `10`
    pub const k_ERatingAgency_DEJUS: Self = Self(10);
    /// `k_ERatingAgency_IMDA` = `11`
    pub const k_ERatingAgency_IMDA: Self = Self(11);
    /// `k_ERatingAgency_FPB` = `12`
    pub const k_ERatingAgency_FPB: Self = Self(12);
    /// `k_ERatingAgency_TESRI` = `13`
    pub const k_ERatingAgency_TESRI: Self = Self(13);
    /// `k_ERatingAgency_RARS` = `14`
    pub const k_ERatingAgency_RARS: Self = Self(14);
    /// `k_ERatingAgency_AGCOM` = `15`
    pub const k_ERatingAgency_AGCOM: Self = Self(15);
    /// `k_ERatingAgency_IGRS` = `16`
    pub const k_ERatingAgency_IGRS: Self = Self(16);
    /// `k_ERatingAgency_Steam_Germany` = `17`
    pub const k_ERatingAgency_Steam_Germany: Self = Self(17);
    /// `k_ERatingAgency_Steam_Australia` = `18`
    pub const k_ERatingAgency_Steam_Australia: Self = Self(18);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ERatingAgency {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
