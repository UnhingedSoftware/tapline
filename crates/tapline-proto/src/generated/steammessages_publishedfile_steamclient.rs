//! Generated from `steammessages_publishedfile.steamclient.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EPublishedFileRevision`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPublishedFileRevision(pub i32);

impl EPublishedFileRevision {
    /// `k_EPublishedFileRevision_Default` = `0`
    pub const k_EPublishedFileRevision_Default: Self = Self(0);
    /// `k_EPublishedFileRevision_Latest` = `1`
    pub const k_EPublishedFileRevision_Latest: Self = Self(1);
    /// `k_EPublishedFileRevision_ApprovedSnapshot` = `2`
    pub const k_EPublishedFileRevision_ApprovedSnapshot: Self = Self(2);
    /// `k_EPublishedFileRevision_ApprovedSnapshot_China` = `3`
    pub const k_EPublishedFileRevision_ApprovedSnapshot_China: Self = Self(3);
    /// `k_EPublishedFileRevision_RejectedSnapshot` = `4`
    pub const k_EPublishedFileRevision_RejectedSnapshot: Self = Self(4);
    /// `k_EPublishedFileRevision_RejectedSnapshot_China` = `5`
    pub const k_EPublishedFileRevision_RejectedSnapshot_China: Self = Self(5);
    /// `k_EPublishedFileRevision_AuthorSnapshot` = `6`
    pub const k_EPublishedFileRevision_AuthorSnapshot: Self = Self(6);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPublishedFileRevision {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPublishedFileForSaleStatus`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPublishedFileForSaleStatus(pub i32);

impl EPublishedFileForSaleStatus {
    /// `k_PFFSS_NotForSale` = `0`
    pub const k_PFFSS_NotForSale: Self = Self(0);
    /// `k_PFFSS_PendingApproval` = `1`
    pub const k_PFFSS_PendingApproval: Self = Self(1);
    /// `k_PFFSS_ApprovedForSale` = `2`
    pub const k_PFFSS_ApprovedForSale: Self = Self(2);
    /// `k_PFFSS_RejectedForSale` = `3`
    pub const k_PFFSS_RejectedForSale: Self = Self(3);
    /// `k_PFFSS_NoLongerForSale` = `4`
    pub const k_PFFSS_NoLongerForSale: Self = Self(4);
    /// `k_PFFSS_TentativeApproval` = `5`
    pub const k_PFFSS_TentativeApproval: Self = Self(5);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPublishedFileForSaleStatus {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EQueryFilesSpecialFilter`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EQueryFilesSpecialFilter(pub i32);

impl EQueryFilesSpecialFilter {
    /// `k_EQueryFilesSpecialFilter_None` = `0`
    pub const k_EQueryFilesSpecialFilter_None: Self = Self(0);
    /// `k_EQueryFilesSpecialFilter_AcceptedForUse` = `1`
    pub const k_EQueryFilesSpecialFilter_AcceptedForUse: Self = Self(1);
    /// `k_EQueryFilesSpecialFilter_FavoritedByFriends` = `2`
    pub const k_EQueryFilesSpecialFilter_FavoritedByFriends: Self = Self(2);
    /// `k_EQueryFilesSpecialFilter_CreateByFriends` = `3`
    pub const k_EQueryFilesSpecialFilter_CreateByFriends: Self = Self(3);
    /// `k_EQueryFilesSpecialFilter_CreatedByFollowed` = `4`
    pub const k_EQueryFilesSpecialFilter_CreatedByFollowed: Self = Self(4);
    /// `k_EQueryFilesSpecialFilter_Reported` = `5`
    pub const k_EQueryFilesSpecialFilter_Reported: Self = Self(5);
    /// `k_EQueryFilesSpecialFilter_ParentItems` = `6`
    pub const k_EQueryFilesSpecialFilter_ParentItems: Self = Self(6);
    /// `k_EQueryFilesSpecialFilter_ParentCollections` = `7`
    pub const k_EQueryFilesSpecialFilter_ParentCollections: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EQueryFilesSpecialFilter {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EQueryFilesSearchTextTarget`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EQueryFilesSearchTextTarget(pub i32);

impl EQueryFilesSearchTextTarget {
    /// `k_EQueryFilesSearchTextTarget_AllText` = `0`
    pub const k_EQueryFilesSearchTextTarget_AllText: Self = Self(0);
    /// `k_EQueryFilesSearchTextTarget_Title` = `1`
    pub const k_EQueryFilesSearchTextTarget_Title: Self = Self(1);
    /// `k_EQueryFilesSearchTextTarget_Description` = `2`
    pub const k_EQueryFilesSearchTextTarget_Description: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EQueryFilesSearchTextTarget {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `CPublishedFile_Vote_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Vote_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub vote_up: Option<bool>,
}

impl Message for CPublishedFile_Vote_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.vote_up = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.vote_up {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// `CPublishedFile_Vote_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Vote_Response {}

impl Message for CPublishedFile_Vote_Response {
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

/// `CPublishedFile_Subscribe_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Subscribe_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub list_type: Option<u32>,
    /// Field 3.
    pub appid: Option<i32>,
    /// Field 4.
    pub notify_client: Option<bool>,
    /// Field 5.
    pub include_dependencies: Option<bool>,
}

impl Message for CPublishedFile_Subscribe_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.list_type = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.appid = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.notify_client = Some(decoder.read_bool()?);
                }
                5 => {
                    self.include_dependencies = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.list_type {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.appid {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.notify_client {
            encoder.write_bool_field(4, *value);
        }
        if let Some(value) = &self.include_dependencies {
            encoder.write_bool_field(5, *value);
        }
    }
}

/// `CPublishedFile_Subscribe_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Subscribe_Response {}

impl Message for CPublishedFile_Subscribe_Response {
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

/// `CPublishedFile_Unsubscribe_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Unsubscribe_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub list_type: Option<u32>,
    /// Field 3.
    pub appid: Option<i32>,
    /// Field 4.
    pub notify_client: Option<bool>,
}

impl Message for CPublishedFile_Unsubscribe_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.list_type = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.appid = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.notify_client = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.list_type {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.appid {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.notify_client {
            encoder.write_bool_field(4, *value);
        }
    }
}

/// `CPublishedFile_Unsubscribe_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Unsubscribe_Response {}

impl Message for CPublishedFile_Unsubscribe_Response {
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

/// `CPublishedFile_CanSubscribe_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_CanSubscribe_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
}

impl Message for CPublishedFile_CanSubscribe_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CPublishedFile_CanSubscribe_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_CanSubscribe_Response {
    /// Field 1.
    pub can_subscribe: Option<bool>,
}

impl Message for CPublishedFile_CanSubscribe_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.can_subscribe = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.can_subscribe {
            encoder.write_bool_field(1, *value);
        }
    }
}

/// `CPublishedFile_GetSubSectionData_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetSubSectionData_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub for_table_of_contents: Option<bool>,
    /// Field 3.
    pub specific_sectionid: Option<u64>,
    /// Field 4.
    pub desired_revision:
        Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
}

impl CPublishedFile_GetSubSectionData_Request {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
}

impl Message for CPublishedFile_GetSubSectionData_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.for_table_of_contents = Some(decoder.read_bool()?);
                }
                3 => {
                    self.specific_sectionid = Some(decoder.read_varint()?);
                }
                4 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.for_table_of_contents {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.specific_sectionid {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
    }
}

/// `PublishedFileSubSection` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PublishedFileSubSection {
    /// Field 1.
    pub sectionid: Option<u64>,
    /// Field 2.
    pub title: Option<String>,
    /// Field 3.
    pub description_text: Option<String>,
    /// Field 4.
    pub sort_order: Option<u32>,
}

impl Message for PublishedFileSubSection {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.sectionid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.title = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.description_text = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.sort_order = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.sectionid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.title {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.description_text {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.sort_order {
            encoder.write_varint_field(4, u64::from(*value));
        }
    }
}

/// `CPublishedFile_GetSubSectionData_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetSubSectionData_Response {
    /// Field 1.
    pub sub_sections: Vec<crate::steammessages_publishedfile_steamclient::PublishedFileSubSection>,
}

impl Message for CPublishedFile_GetSubSectionData_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.sub_sections.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileSubSection::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.sub_sections {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CPublishedFile_Publish_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Publish_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub consumer_appid: Option<u32>,
    /// Field 3.
    pub cloudfilename: Option<String>,
    /// Field 4.
    pub preview_cloudfilename: Option<String>,
    /// Field 5.
    pub title: Option<String>,
    /// Field 6.
    pub file_description: Option<String>,
    /// Field 7.
    pub file_type: Option<u32>,
    /// Field 8.
    pub consumer_shortcut_name: Option<String>,
    /// Field 9.
    pub youtube_username: Option<String>,
    /// Field 10.
    pub youtube_videoid: Option<String>,
    /// Field 11.
    pub visibility: Option<u32>,
    /// Field 12.
    pub redirect_uri: Option<String>,
    /// Field 13.
    pub tags: Vec<String>,
    /// Field 14.
    pub collection_type: Option<String>,
    /// Field 15.
    pub game_type: Option<String>,
    /// Field 16.
    pub url: Option<String>,
}

impl Message for CPublishedFile_Publish_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.consumer_appid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.cloudfilename = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.preview_cloudfilename = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.title = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.file_description = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.file_type = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.consumer_shortcut_name = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.youtube_username = Some(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.youtube_videoid = Some(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.visibility = Some(decoder.read_varint()? as u32);
                }
                12 => {
                    self.redirect_uri = Some(decoder.read_string()?.to_owned());
                }
                13 => {
                    self.tags.push(decoder.read_string()?.to_owned());
                }
                14 => {
                    self.collection_type = Some(decoder.read_string()?.to_owned());
                }
                15 => {
                    self.game_type = Some(decoder.read_string()?.to_owned());
                }
                16 => {
                    self.url = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.consumer_appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.cloudfilename {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.preview_cloudfilename {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.title {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.file_description {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.file_type {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.consumer_shortcut_name {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.youtube_username {
            encoder.write_string_field(9, value);
        }
        if let Some(value) = &self.youtube_videoid {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.visibility {
            encoder.write_varint_field(11, u64::from(*value));
        }
        if let Some(value) = &self.redirect_uri {
            encoder.write_string_field(12, value);
        }
        for value in &self.tags {
            encoder.write_string_field(13, value);
        }
        if let Some(value) = &self.collection_type {
            encoder.write_string_field(14, value);
        }
        if let Some(value) = &self.game_type {
            encoder.write_string_field(15, value);
        }
        if let Some(value) = &self.url {
            encoder.write_string_field(16, value);
        }
    }
}

/// `CPublishedFile_Publish_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Publish_Response {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub redirect_uri: Option<String>,
}

impl Message for CPublishedFile_Publish_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.redirect_uri = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.redirect_uri {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CPublishedFile_GetDetails_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetDetails_Request {
    /// Field 1.
    pub publishedfileids: Vec<u64>,
    /// Field 2.
    pub includetags: Option<bool>,
    /// Field 3.
    pub includeadditionalpreviews: Option<bool>,
    /// Field 4.
    pub includechildren: Option<bool>,
    /// Field 5.
    pub includekvtags: Option<bool>,
    /// Field 6.
    pub includevotes: Option<bool>,
    /// Field 8.
    pub short_description: Option<bool>,
    /// Field 10.
    pub includeforsaledata: Option<bool>,
    /// Field 11.
    pub includemetadata: Option<bool>,
    /// Field 12.
    pub language: Option<i32>,
    /// Field 13.
    pub return_playtime_stats: Option<u32>,
    /// Field 14.
    pub appid: Option<u32>,
    /// Field 15.
    pub strip_description_bbcode: Option<bool>,
    /// Field 16.
    pub desired_revision:
        Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 17.
    pub includereactions: Option<bool>,
    /// Field 18.
    pub admin_query: Option<bool>,
}

impl CPublishedFile_GetDetails_Request {
    /// Field 12 , or its schema default when absent.
    #[must_use]
    pub fn language_or_default(&self) -> i32 {
        self.language.unwrap_or(0_i32)
    }
    /// Field 16 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
    /// Field 17 , or its schema default when absent.
    #[must_use]
    pub fn includereactions_or_default(&self) -> bool {
        self.includereactions.unwrap_or(false)
    }
}

impl Message for CPublishedFile_GetDetails_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.publishedfileids,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                2 => {
                    self.includetags = Some(decoder.read_bool()?);
                }
                3 => {
                    self.includeadditionalpreviews = Some(decoder.read_bool()?);
                }
                4 => {
                    self.includechildren = Some(decoder.read_bool()?);
                }
                5 => {
                    self.includekvtags = Some(decoder.read_bool()?);
                }
                6 => {
                    self.includevotes = Some(decoder.read_bool()?);
                }
                8 => {
                    self.short_description = Some(decoder.read_bool()?);
                }
                10 => {
                    self.includeforsaledata = Some(decoder.read_bool()?);
                }
                11 => {
                    self.includemetadata = Some(decoder.read_bool()?);
                }
                12 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                13 => {
                    self.return_playtime_stats = Some(decoder.read_varint()? as u32);
                }
                14 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                15 => {
                    self.strip_description_bbcode = Some(decoder.read_bool()?);
                }
                16 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                17 => {
                    self.includereactions = Some(decoder.read_bool()?);
                }
                18 => {
                    self.admin_query = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.publishedfileids {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.includetags {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.includeadditionalpreviews {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.includechildren {
            encoder.write_bool_field(4, *value);
        }
        if let Some(value) = &self.includekvtags {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.includevotes {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.short_description {
            encoder.write_bool_field(8, *value);
        }
        if let Some(value) = &self.includeforsaledata {
            encoder.write_bool_field(10, *value);
        }
        if let Some(value) = &self.includemetadata {
            encoder.write_bool_field(11, *value);
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(12, *value);
        }
        if let Some(value) = &self.return_playtime_stats {
            encoder.write_varint_field(13, u64::from(*value));
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(14, u64::from(*value));
        }
        if let Some(value) = &self.strip_description_bbcode {
            encoder.write_bool_field(15, *value);
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(16, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.includereactions {
            encoder.write_bool_field(17, *value);
        }
        if let Some(value) = &self.admin_query {
            encoder.write_bool_field(18, *value);
        }
    }
}

/// `PublishedFileAuthorSnapshot` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PublishedFileAuthorSnapshot {
    /// Field 1.
    pub timestamp: Option<u32>,
    /// Field 2.
    pub game_branch_min: Option<String>,
    /// Field 3.
    pub game_branch_max: Option<String>,
    /// Field 4.
    pub manifestid: Option<u64>,
}

impl Message for PublishedFileAuthorSnapshot {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.timestamp = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.game_branch_min = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.game_branch_max = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.manifestid = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.timestamp {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.game_branch_min {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.game_branch_max {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.manifestid {
            encoder.write_fixed64_field(4, *value);
        }
    }
}

/// Types nested inside [`PublishedFileDetails`].
pub mod published_file_details {
    use super::*;

    /// `Tag` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Tag {
        /// Field 1.
        pub tag: Option<String>,
        /// Field 2.
        pub adminonly: Option<bool>,
        /// Field 3.
        pub display_name: Option<String>,
    }

    impl Message for Tag {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.tag = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.adminonly = Some(decoder.read_bool()?);
                    }
                    3 => {
                        self.display_name = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.tag {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.adminonly {
                encoder.write_bool_field(2, *value);
            }
            if let Some(value) = &self.display_name {
                encoder.write_string_field(3, value);
            }
        }
    }

    /// `Preview` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Preview {
        /// Field 1.
        pub previewid: Option<u64>,
        /// Field 2.
        pub sortorder: Option<u32>,
        /// Field 3.
        pub url: Option<String>,
        /// Field 4.
        pub size: Option<u32>,
        /// Field 5.
        pub filename: Option<String>,
        /// Field 6.
        pub youtubevideoid: Option<String>,
        /// Field 7.
        pub preview_type: Option<u32>,
        /// Field 8.
        pub external_reference: Option<String>,
    }

    impl Message for Preview {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.previewid = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.sortorder = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.url = Some(decoder.read_string()?.to_owned());
                    }
                    4 => {
                        self.size = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.filename = Some(decoder.read_string()?.to_owned());
                    }
                    6 => {
                        self.youtubevideoid = Some(decoder.read_string()?.to_owned());
                    }
                    7 => {
                        self.preview_type = Some(decoder.read_varint()? as u32);
                    }
                    8 => {
                        self.external_reference = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.previewid {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.sortorder {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.url {
                encoder.write_string_field(3, value);
            }
            if let Some(value) = &self.size {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.filename {
                encoder.write_string_field(5, value);
            }
            if let Some(value) = &self.youtubevideoid {
                encoder.write_string_field(6, value);
            }
            if let Some(value) = &self.preview_type {
                encoder.write_varint_field(7, u64::from(*value));
            }
            if let Some(value) = &self.external_reference {
                encoder.write_string_field(8, value);
            }
        }
    }

    /// `Child` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Child {
        /// Field 1.
        pub publishedfileid: Option<u64>,
        /// Field 2.
        pub sortorder: Option<u32>,
        /// Field 3.
        pub file_type: Option<u32>,
    }

    impl Message for Child {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.publishedfileid = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.sortorder = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.file_type = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.publishedfileid {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.sortorder {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.file_type {
                encoder.write_varint_field(3, u64::from(*value));
            }
        }
    }

    /// `KVTag` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct KVTag {
        /// Field 1.
        pub key: Option<String>,
        /// Field 2.
        pub value: Option<String>,
    }

    impl Message for KVTag {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.key = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.value = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.key {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.value {
                encoder.write_string_field(2, value);
            }
        }
    }

    /// `VoteData` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct VoteData {
        /// Field 1.
        pub score: Option<f32>,
        /// Field 2.
        pub votes_up: Option<u32>,
        /// Field 3.
        pub votes_down: Option<u32>,
        /// Field 4.
        pub trusted_score: Option<f32>,
        /// Field 5.
        pub trusted_votes_up: Option<u32>,
        /// Field 6.
        pub trusted_votes_down: Option<u32>,
    }

    impl Message for VoteData {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.score = Some(decoder.read_float()?);
                    }
                    2 => {
                        self.votes_up = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.votes_down = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.trusted_score = Some(decoder.read_float()?);
                    }
                    5 => {
                        self.trusted_votes_up = Some(decoder.read_varint()? as u32);
                    }
                    6 => {
                        self.trusted_votes_down = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.score {
                encoder.write_float_field(1, *value);
            }
            if let Some(value) = &self.votes_up {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.votes_down {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.trusted_score {
                encoder.write_float_field(4, *value);
            }
            if let Some(value) = &self.trusted_votes_up {
                encoder.write_varint_field(5, u64::from(*value));
            }
            if let Some(value) = &self.trusted_votes_down {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }

    /// `ForSaleData` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ForSaleData {
        /// Field 1.
        pub is_for_sale: Option<bool>,
        /// Field 2.
        pub price_category: Option<u32>,
        /// Field 3.
        pub estatus:
            Option<crate::steammessages_publishedfile_steamclient::EPublishedFileForSaleStatus>,
        /// Field 4.
        pub price_category_floor: Option<u32>,
        /// Field 5.
        pub price_is_pay_what_you_want: Option<bool>,
        /// Field 6.
        pub discount_percentage: Option<u32>,
    }

    impl ForSaleData {
        /// Field 3 , or its schema default when absent.
        #[must_use]
        pub fn estatus_or_default(
            &self,
        ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileForSaleStatus {
            self.estatus.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileForSaleStatus::k_PFFSS_NotForSale)
        }
    }

    impl Message for ForSaleData {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.is_for_sale = Some(decoder.read_bool()?);
                    }
                    2 => {
                        self.price_category = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.estatus = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileForSaleStatus::from(decoder.read_varint()? as i32));
                    }
                    4 => {
                        self.price_category_floor = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.price_is_pay_what_you_want = Some(decoder.read_bool()?);
                    }
                    6 => {
                        self.discount_percentage = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.is_for_sale {
                encoder.write_bool_field(1, *value);
            }
            if let Some(value) = &self.price_category {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.estatus {
                encoder.write_varint_field(3, i64::from(value.value()) as u64);
            }
            if let Some(value) = &self.price_category_floor {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.price_is_pay_what_you_want {
                encoder.write_bool_field(5, *value);
            }
            if let Some(value) = &self.discount_percentage {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }

    /// `PlaytimeStats` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PlaytimeStats {
        /// Field 1.
        pub playtime_seconds: Option<u64>,
        /// Field 2.
        pub num_sessions: Option<u64>,
    }

    impl Message for PlaytimeStats {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.playtime_seconds = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.num_sessions = Some(decoder.read_varint()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.playtime_seconds {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.num_sessions {
                encoder.write_varint_field(2, *value);
            }
        }
    }

    /// `Reaction` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Reaction {
        /// Field 1.
        pub reactionid: Option<u32>,
        /// Field 2.
        pub count: Option<u32>,
    }

    impl Message for Reaction {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.reactionid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.count = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.reactionid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.count {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `PublishedFileDetails` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PublishedFileDetails {
    /// Field 1.
    pub result: Option<u32>,
    /// Field 2.
    pub publishedfileid: Option<u64>,
    /// Field 3.
    pub creator: Option<u64>,
    /// Field 4.
    pub creator_appid: Option<u32>,
    /// Field 5.
    pub consumer_appid: Option<u32>,
    /// Field 6.
    pub consumer_shortcutid: Option<u32>,
    /// Field 7.
    pub filename: Option<String>,
    /// Field 8.
    pub file_size: Option<u64>,
    /// Field 9.
    pub preview_file_size: Option<u64>,
    /// Field 10.
    pub file_url: Option<String>,
    /// Field 11.
    pub preview_url: Option<String>,
    /// Field 12.
    pub youtubevideoid: Option<String>,
    /// Field 13.
    pub url: Option<String>,
    /// Field 14.
    pub hcontent_file: Option<u64>,
    /// Field 15.
    pub hcontent_preview: Option<u64>,
    /// Field 16.
    pub title: Option<String>,
    /// Field 17.
    pub file_description: Option<String>,
    /// Field 18.
    pub short_description: Option<String>,
    /// Field 19.
    pub time_created: Option<u32>,
    /// Field 20.
    pub time_updated: Option<u32>,
    /// Field 21.
    pub visibility: Option<u32>,
    /// Field 22.
    pub flags: Option<u32>,
    /// Field 23.
    pub workshop_file: Option<bool>,
    /// Field 24.
    pub workshop_accepted: Option<bool>,
    /// Field 25.
    pub show_subscribe_all: Option<bool>,
    /// Field 26.
    pub num_comments_developer: Option<i32>,
    /// Field 27.
    pub num_comments_public: Option<i32>,
    /// Field 28.
    pub banned: Option<bool>,
    /// Field 29.
    pub ban_reason: Option<String>,
    /// Field 30.
    pub banner: Option<u64>,
    /// Field 31.
    pub can_be_deleted: Option<bool>,
    /// Field 32.
    pub incompatible: Option<bool>,
    /// Field 33.
    pub app_name: Option<String>,
    /// Field 34.
    pub file_type: Option<u32>,
    /// Field 35.
    pub can_subscribe: Option<bool>,
    /// Field 36.
    pub subscriptions: Option<u32>,
    /// Field 37.
    pub favorited: Option<u32>,
    /// Field 38.
    pub followers: Option<u32>,
    /// Field 39.
    pub lifetime_subscriptions: Option<u32>,
    /// Field 40.
    pub lifetime_favorited: Option<u32>,
    /// Field 41.
    pub lifetime_followers: Option<u32>,
    /// Field 62.
    pub lifetime_playtime: Option<u64>,
    /// Field 63.
    pub lifetime_playtime_sessions: Option<u64>,
    /// Field 42.
    pub views: Option<u32>,
    /// Field 43.
    pub image_width: Option<u32>,
    /// Field 44.
    pub image_height: Option<u32>,
    /// Field 45.
    pub image_url: Option<String>,
    /// Field 46.
    pub spoiler_tag: Option<bool>,
    /// Field 47.
    pub shortcutid: Option<u32>,
    /// Field 48.
    pub shortcutname: Option<String>,
    /// Field 49.
    pub num_children: Option<u32>,
    /// Field 50.
    pub num_reports: Option<u32>,
    /// Field 51.
    pub previews:
        Vec<crate::steammessages_publishedfile_steamclient::published_file_details::Preview>,
    /// Field 52.
    pub tags: Vec<crate::steammessages_publishedfile_steamclient::published_file_details::Tag>,
    /// Field 53.
    pub children:
        Vec<crate::steammessages_publishedfile_steamclient::published_file_details::Child>,
    /// Field 54.
    pub kvtags: Vec<crate::steammessages_publishedfile_steamclient::published_file_details::KVTag>,
    /// Field 55.
    pub vote_data:
        Option<crate::steammessages_publishedfile_steamclient::published_file_details::VoteData>,
    /// Field 64.
    pub playtime_stats: Option<
        crate::steammessages_publishedfile_steamclient::published_file_details::PlaytimeStats,
    >,
    /// Field 56.
    pub time_subscribed: Option<u32>,
    /// Field 57.
    pub for_sale_data:
        Option<crate::steammessages_publishedfile_steamclient::published_file_details::ForSaleData>,
    /// Field 58.
    pub metadata: Option<String>,
    /// Field 61.
    pub language: Option<i32>,
    /// Field 65.
    pub maybe_inappropriate_sex: Option<bool>,
    /// Field 66.
    pub maybe_inappropriate_violence: Option<bool>,
    /// Field 72.
    pub content_descriptorids: Vec<crate::enums_productinfo::EContentDescriptorID>,
    /// Field 67.
    pub revision_change_number: Option<u64>,
    /// Field 68.
    pub revision: Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 69.
    pub available_revisions:
        Vec<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 70.
    pub reactions:
        Vec<crate::steammessages_publishedfile_steamclient::published_file_details::Reaction>,
    /// Field 71.
    pub ban_text_check_result: Option<crate::steammessages_base::EBanContentCheckResult>,
    /// Field 73.
    pub search_score: Option<f32>,
    /// Field 74.
    pub external_asset_id: Option<u64>,
    /// Field 75.
    pub author_snapshots:
        Vec<crate::steammessages_publishedfile_steamclient::PublishedFileAuthorSnapshot>,
}

impl PublishedFileDetails {
    /// Field 61 , or its schema default when absent.
    #[must_use]
    pub fn language_or_default(&self) -> i32 {
        self.language.unwrap_or(0_i32)
    }
    /// Field 68 , or its schema default when absent.
    #[must_use]
    pub fn revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
    /// Field 71 , or its schema default when absent.
    #[must_use]
    pub fn ban_text_check_result_or_default(
        &self,
    ) -> crate::steammessages_base::EBanContentCheckResult {
        self.ban_text_check_result.unwrap_or(
            crate::steammessages_base::EBanContentCheckResult::k_EBanContentCheckResult_NotScanned,
        )
    }
}

impl Message for PublishedFileDetails {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => { self.result = Some(decoder.read_varint()? as u32); }
                2 => { self.publishedfileid = Some(decoder.read_varint()?); }
                3 => { self.creator = Some(decoder.read_fixed64()?); }
                4 => { self.creator_appid = Some(decoder.read_varint()? as u32); }
                5 => { self.consumer_appid = Some(decoder.read_varint()? as u32); }
                6 => { self.consumer_shortcutid = Some(decoder.read_varint()? as u32); }
                7 => { self.filename = Some(decoder.read_string()?.to_owned()); }
                8 => { self.file_size = Some(decoder.read_varint()?); }
                9 => { self.preview_file_size = Some(decoder.read_varint()?); }
                10 => { self.file_url = Some(decoder.read_string()?.to_owned()); }
                11 => { self.preview_url = Some(decoder.read_string()?.to_owned()); }
                12 => { self.youtubevideoid = Some(decoder.read_string()?.to_owned()); }
                13 => { self.url = Some(decoder.read_string()?.to_owned()); }
                14 => { self.hcontent_file = Some(decoder.read_fixed64()?); }
                15 => { self.hcontent_preview = Some(decoder.read_fixed64()?); }
                16 => { self.title = Some(decoder.read_string()?.to_owned()); }
                17 => { self.file_description = Some(decoder.read_string()?.to_owned()); }
                18 => { self.short_description = Some(decoder.read_string()?.to_owned()); }
                19 => { self.time_created = Some(decoder.read_varint()? as u32); }
                20 => { self.time_updated = Some(decoder.read_varint()? as u32); }
                21 => { self.visibility = Some(decoder.read_varint()? as u32); }
                22 => { self.flags = Some(decoder.read_varint()? as u32); }
                23 => { self.workshop_file = Some(decoder.read_bool()?); }
                24 => { self.workshop_accepted = Some(decoder.read_bool()?); }
                25 => { self.show_subscribe_all = Some(decoder.read_bool()?); }
                26 => { self.num_comments_developer = Some(decoder.read_varint()? as i32); }
                27 => { self.num_comments_public = Some(decoder.read_varint()? as i32); }
                28 => { self.banned = Some(decoder.read_bool()?); }
                29 => { self.ban_reason = Some(decoder.read_string()?.to_owned()); }
                30 => { self.banner = Some(decoder.read_fixed64()?); }
                31 => { self.can_be_deleted = Some(decoder.read_bool()?); }
                32 => { self.incompatible = Some(decoder.read_bool()?); }
                33 => { self.app_name = Some(decoder.read_string()?.to_owned()); }
                34 => { self.file_type = Some(decoder.read_varint()? as u32); }
                35 => { self.can_subscribe = Some(decoder.read_bool()?); }
                36 => { self.subscriptions = Some(decoder.read_varint()? as u32); }
                37 => { self.favorited = Some(decoder.read_varint()? as u32); }
                38 => { self.followers = Some(decoder.read_varint()? as u32); }
                39 => { self.lifetime_subscriptions = Some(decoder.read_varint()? as u32); }
                40 => { self.lifetime_favorited = Some(decoder.read_varint()? as u32); }
                41 => { self.lifetime_followers = Some(decoder.read_varint()? as u32); }
                62 => { self.lifetime_playtime = Some(decoder.read_varint()?); }
                63 => { self.lifetime_playtime_sessions = Some(decoder.read_varint()?); }
                42 => { self.views = Some(decoder.read_varint()? as u32); }
                43 => { self.image_width = Some(decoder.read_varint()? as u32); }
                44 => { self.image_height = Some(decoder.read_varint()? as u32); }
                45 => { self.image_url = Some(decoder.read_string()?.to_owned()); }
                46 => { self.spoiler_tag = Some(decoder.read_bool()?); }
                47 => { self.shortcutid = Some(decoder.read_varint()? as u32); }
                48 => { self.shortcutname = Some(decoder.read_string()?.to_owned()); }
                49 => { self.num_children = Some(decoder.read_varint()? as u32); }
                50 => { self.num_reports = Some(decoder.read_varint()? as u32); }
                51 => { self.previews.push({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::Preview::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                52 => { self.tags.push({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::Tag::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                53 => { self.children.push({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::Child::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                54 => { self.kvtags.push({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::KVTag::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                55 => { self.vote_data = Some({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::VoteData::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                64 => { self.playtime_stats = Some({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::PlaytimeStats::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                56 => { self.time_subscribed = Some(decoder.read_varint()? as u32); }
                57 => { self.for_sale_data = Some({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::ForSaleData::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                58 => { self.metadata = Some(decoder.read_string()?.to_owned()); }
                61 => { self.language = Some(decoder.read_varint()? as i32); }
                65 => { self.maybe_inappropriate_sex = Some(decoder.read_bool()?); }
                66 => { self.maybe_inappropriate_violence = Some(decoder.read_bool()?); }
                72 => decoder.read_maybe_packed(key.wire_type, &mut self.content_descriptorids, |d: &mut Decoder<'_>| Ok(crate::enums_productinfo::EContentDescriptorID::from(d.read_varint()? as i32)))?,
                67 => { self.revision_change_number = Some(decoder.read_varint()?); }
                68 => { self.revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32)); }
                69 => decoder.read_maybe_packed(key.wire_type, &mut self.available_revisions, |d: &mut Decoder<'_>| Ok(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(d.read_varint()? as i32)))?,
                70 => { self.reactions.push({ let mut nested = crate::steammessages_publishedfile_steamclient::published_file_details::Reaction::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                71 => { self.ban_text_check_result = Some(crate::steammessages_base::EBanContentCheckResult::from(decoder.read_varint()? as i32)); }
                73 => { self.search_score = Some(decoder.read_float()?); }
                74 => { self.external_asset_id = Some(decoder.read_varint()?); }
                75 => { self.author_snapshots.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileAuthorSnapshot::default(); decoder.read_nested(|d| nested.merge(d))?; nested }); }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.creator {
            encoder.write_fixed64_field(3, *value);
        }
        if let Some(value) = &self.creator_appid {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.consumer_appid {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.consumer_shortcutid {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.filename {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.file_size {
            encoder.write_varint_field(8, *value);
        }
        if let Some(value) = &self.preview_file_size {
            encoder.write_varint_field(9, *value);
        }
        if let Some(value) = &self.file_url {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.preview_url {
            encoder.write_string_field(11, value);
        }
        if let Some(value) = &self.youtubevideoid {
            encoder.write_string_field(12, value);
        }
        if let Some(value) = &self.url {
            encoder.write_string_field(13, value);
        }
        if let Some(value) = &self.hcontent_file {
            encoder.write_fixed64_field(14, *value);
        }
        if let Some(value) = &self.hcontent_preview {
            encoder.write_fixed64_field(15, *value);
        }
        if let Some(value) = &self.title {
            encoder.write_string_field(16, value);
        }
        if let Some(value) = &self.file_description {
            encoder.write_string_field(17, value);
        }
        if let Some(value) = &self.short_description {
            encoder.write_string_field(18, value);
        }
        if let Some(value) = &self.time_created {
            encoder.write_varint_field(19, u64::from(*value));
        }
        if let Some(value) = &self.time_updated {
            encoder.write_varint_field(20, u64::from(*value));
        }
        if let Some(value) = &self.visibility {
            encoder.write_varint_field(21, u64::from(*value));
        }
        if let Some(value) = &self.flags {
            encoder.write_varint_field(22, u64::from(*value));
        }
        if let Some(value) = &self.workshop_file {
            encoder.write_bool_field(23, *value);
        }
        if let Some(value) = &self.workshop_accepted {
            encoder.write_bool_field(24, *value);
        }
        if let Some(value) = &self.show_subscribe_all {
            encoder.write_bool_field(25, *value);
        }
        if let Some(value) = &self.num_comments_developer {
            encoder.write_int32_field(26, *value);
        }
        if let Some(value) = &self.num_comments_public {
            encoder.write_int32_field(27, *value);
        }
        if let Some(value) = &self.banned {
            encoder.write_bool_field(28, *value);
        }
        if let Some(value) = &self.ban_reason {
            encoder.write_string_field(29, value);
        }
        if let Some(value) = &self.banner {
            encoder.write_fixed64_field(30, *value);
        }
        if let Some(value) = &self.can_be_deleted {
            encoder.write_bool_field(31, *value);
        }
        if let Some(value) = &self.incompatible {
            encoder.write_bool_field(32, *value);
        }
        if let Some(value) = &self.app_name {
            encoder.write_string_field(33, value);
        }
        if let Some(value) = &self.file_type {
            encoder.write_varint_field(34, u64::from(*value));
        }
        if let Some(value) = &self.can_subscribe {
            encoder.write_bool_field(35, *value);
        }
        if let Some(value) = &self.subscriptions {
            encoder.write_varint_field(36, u64::from(*value));
        }
        if let Some(value) = &self.favorited {
            encoder.write_varint_field(37, u64::from(*value));
        }
        if let Some(value) = &self.followers {
            encoder.write_varint_field(38, u64::from(*value));
        }
        if let Some(value) = &self.lifetime_subscriptions {
            encoder.write_varint_field(39, u64::from(*value));
        }
        if let Some(value) = &self.lifetime_favorited {
            encoder.write_varint_field(40, u64::from(*value));
        }
        if let Some(value) = &self.lifetime_followers {
            encoder.write_varint_field(41, u64::from(*value));
        }
        if let Some(value) = &self.lifetime_playtime {
            encoder.write_varint_field(62, *value);
        }
        if let Some(value) = &self.lifetime_playtime_sessions {
            encoder.write_varint_field(63, *value);
        }
        if let Some(value) = &self.views {
            encoder.write_varint_field(42, u64::from(*value));
        }
        if let Some(value) = &self.image_width {
            encoder.write_varint_field(43, u64::from(*value));
        }
        if let Some(value) = &self.image_height {
            encoder.write_varint_field(44, u64::from(*value));
        }
        if let Some(value) = &self.image_url {
            encoder.write_string_field(45, value);
        }
        if let Some(value) = &self.spoiler_tag {
            encoder.write_bool_field(46, *value);
        }
        if let Some(value) = &self.shortcutid {
            encoder.write_varint_field(47, u64::from(*value));
        }
        if let Some(value) = &self.shortcutname {
            encoder.write_string_field(48, value);
        }
        if let Some(value) = &self.num_children {
            encoder.write_varint_field(49, u64::from(*value));
        }
        if let Some(value) = &self.num_reports {
            encoder.write_varint_field(50, u64::from(*value));
        }
        for value in &self.previews {
            encoder.write_message_field(51, value);
        }
        for value in &self.tags {
            encoder.write_message_field(52, value);
        }
        for value in &self.children {
            encoder.write_message_field(53, value);
        }
        for value in &self.kvtags {
            encoder.write_message_field(54, value);
        }
        if let Some(value) = &self.vote_data {
            encoder.write_message_field(55, value);
        }
        if let Some(value) = &self.playtime_stats {
            encoder.write_message_field(64, value);
        }
        if let Some(value) = &self.time_subscribed {
            encoder.write_varint_field(56, u64::from(*value));
        }
        if let Some(value) = &self.for_sale_data {
            encoder.write_message_field(57, value);
        }
        if let Some(value) = &self.metadata {
            encoder.write_string_field(58, value);
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(61, *value);
        }
        if let Some(value) = &self.maybe_inappropriate_sex {
            encoder.write_bool_field(65, *value);
        }
        if let Some(value) = &self.maybe_inappropriate_violence {
            encoder.write_bool_field(66, *value);
        }
        for value in &self.content_descriptorids {
            encoder.write_varint_field(72, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.revision_change_number {
            encoder.write_varint_field(67, *value);
        }
        if let Some(value) = &self.revision {
            encoder.write_varint_field(68, i64::from(value.value()) as u64);
        }
        for value in &self.available_revisions {
            encoder.write_varint_field(69, i64::from(value.value()) as u64);
        }
        for value in &self.reactions {
            encoder.write_message_field(70, value);
        }
        if let Some(value) = &self.ban_text_check_result {
            encoder.write_varint_field(71, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.search_score {
            encoder.write_float_field(73, *value);
        }
        if let Some(value) = &self.external_asset_id {
            encoder.write_varint_field(74, *value);
        }
        for value in &self.author_snapshots {
            encoder.write_message_field(75, value);
        }
    }
}

/// `CPublishedFile_GetDetails_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetDetails_Response {
    /// Field 1.
    pub publishedfiledetails:
        Vec<crate::steammessages_publishedfile_steamclient::PublishedFileDetails>,
}

impl Message for CPublishedFile_GetDetails_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfiledetails.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileDetails::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.publishedfiledetails {
            encoder.write_message_field(1, value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetItemInfo_Request`].
pub mod c_published_file_get_item_info_request {
    use super::*;

    /// `WorkshopItem` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct WorkshopItem {
        /// Field 1.
        pub published_file_id: Option<u64>,
        /// Field 2.
        pub time_updated: Option<u32>,
        /// Field 3.
        pub desired_revision:
            Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    }

    impl WorkshopItem {
        /// Field 3 , or its schema default when absent.
        #[must_use]
        pub fn desired_revision_or_default(
            &self,
        ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
            self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
        }
    }

    impl Message for WorkshopItem {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.time_updated = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.published_file_id {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.time_updated {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.desired_revision {
                encoder.write_varint_field(3, i64::from(value.value()) as u64);
            }
        }
    }
}

/// `CPublishedFile_GetItemInfo_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetItemInfo_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub last_time_updated: Option<u32>,
    /// Field 3.
    pub workshop_items: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_item_info_request::WorkshopItem>,
    /// Field 4.
    pub desired_revision: Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 5.
    pub full_reconcile: Option<bool>,
}

impl CPublishedFile_GetItemInfo_Request {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
}

impl Message for CPublishedFile_GetItemInfo_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.last_time_updated = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.workshop_items.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_item_info_request::WorkshopItem::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                4 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                5 => {
                    self.full_reconcile = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.last_time_updated {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.workshop_items {
            encoder.write_message_field(3, value);
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.full_reconcile {
            encoder.write_bool_field(5, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetItemInfo_Response`].
pub mod c_published_file_get_item_info_response {
    use super::*;

    /// `WorkshopItemInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct WorkshopItemInfo {
        /// Field 1.
        pub published_file_id: Option<u64>,
        /// Field 2.
        pub time_updated: Option<u32>,
        /// Field 3.
        pub manifest_id: Option<u64>,
        /// Field 4.
        pub flags: Option<u32>,
        /// Field 5.
        pub revision:
            Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
        /// Field 6.
        pub author_snapshots:
            Vec<crate::steammessages_publishedfile_steamclient::PublishedFileAuthorSnapshot>,
    }

    impl WorkshopItemInfo {
        /// Field 5 , or its schema default when absent.
        #[must_use]
        pub fn revision_or_default(
            &self,
        ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
            self.revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
        }
    }

    impl Message for WorkshopItemInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.time_updated = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.manifest_id = Some(decoder.read_fixed64()?);
                    }
                    4 => {
                        self.flags = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                    }
                    6 => {
                        self.author_snapshots.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileAuthorSnapshot::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.published_file_id {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.time_updated {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.manifest_id {
                encoder.write_fixed64_field(3, *value);
            }
            if let Some(value) = &self.flags {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.revision {
                encoder.write_varint_field(5, i64::from(value.value()) as u64);
            }
            for value in &self.author_snapshots {
                encoder.write_message_field(6, value);
            }
        }
    }
}

/// `CPublishedFile_GetItemInfo_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetItemInfo_Response {
    /// Field 1.
    pub update_time: Option<u32>,
    /// Field 2.
    pub workshop_items: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_item_info_response::WorkshopItemInfo>,
    /// Field 3.
    pub private_items: Vec<u64>,
}

impl Message for CPublishedFile_GetItemInfo_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.update_time = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.workshop_items.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_item_info_response::WorkshopItemInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.private_items,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.update_time {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.workshop_items {
            encoder.write_message_field(2, value);
        }
        for value in &self.private_items {
            encoder.write_fixed64_field(3, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetUserFiles_Request`].
pub mod c_published_file_get_user_files_request {
    use super::*;

    /// `KVTag` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct KVTag {
        /// Field 1.
        pub key: Option<String>,
        /// Field 2.
        pub value: Option<String>,
    }

    impl Message for KVTag {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.key = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.value = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.key {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.value {
                encoder.write_string_field(2, value);
            }
        }
    }

    /// `TagGroup` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct TagGroup {
        /// Field 1.
        pub tags: Vec<String>,
    }

    impl Message for TagGroup {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.tags.push(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            for value in &self.tags {
                encoder.write_string_field(1, value);
            }
        }
    }

    /// `DateRange` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct DateRange {
        /// Field 1.
        pub timestamp_start: Option<u32>,
        /// Field 2.
        pub timestamp_end: Option<u32>,
    }

    impl Message for DateRange {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.timestamp_start = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.timestamp_end = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.timestamp_start {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.timestamp_end {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CPublishedFile_GetUserFiles_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetUserFiles_Request {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub appid: Option<u32>,
    /// Field 3.
    pub shortcutid: Option<u32>,
    /// Field 4.
    pub page: Option<u32>,
    /// Field 5.
    pub numperpage: Option<u32>,
    /// Field 6.
    pub r#type: Option<String>,
    /// Field 7.
    pub sortmethod: Option<String>,
    /// Field 9.
    pub privacy: Option<u32>,
    /// Field 10.
    pub requiredtags: Vec<String>,
    /// Field 11.
    pub excludedtags: Vec<String>,
    /// Field 30.
    pub required_kv_tags: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::KVTag>,
    /// Field 14.
    pub filetype: Option<u32>,
    /// Field 15.
    pub creator_appid: Option<u32>,
    /// Field 16.
    pub match_cloud_filename: Option<String>,
    /// Field 27.
    pub cache_max_age_seconds: Option<u32>,
    /// Field 29.
    pub language: Option<i32>,
    /// Field 34.
    pub taggroups: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::TagGroup>,
    /// Field 39.
    pub date_range_created: Option<crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::DateRange>,
    /// Field 40.
    pub date_range_updated: Option<crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::DateRange>,
    /// Field 37.
    pub excluded_content_descriptors: Vec<crate::enums_productinfo::EContentDescriptorID>,
    /// Field 38.
    pub admin_query: Option<bool>,
    /// Field 17.
    pub totalonly: Option<bool>,
    /// Field 18.
    pub ids_only: Option<bool>,
    /// Field 19.
    pub return_vote_data: Option<bool>,
    /// Field 20.
    pub return_tags: Option<bool>,
    /// Field 21.
    pub return_kv_tags: Option<bool>,
    /// Field 22.
    pub return_previews: Option<bool>,
    /// Field 23.
    pub return_children: Option<bool>,
    /// Field 24.
    pub return_short_description: Option<bool>,
    /// Field 26.
    pub return_for_sale_data: Option<bool>,
    /// Field 28.
    pub return_metadata: Option<bool>,
    /// Field 31.
    pub return_playtime_stats: Option<u32>,
    /// Field 32.
    pub strip_description_bbcode: Option<bool>,
    /// Field 35.
    pub return_reactions: Option<bool>,
    /// Field 25.
    pub startindex_override: Option<u32>,
    /// Field 33.
    pub desired_revision: Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 36.
    pub return_apps: Option<bool>,
}

impl CPublishedFile_GetUserFiles_Request {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn page_or_default(&self) -> u32 {
        self.page.unwrap_or(1_u32)
    }
    /// Field 5 , or its schema default when absent.
    #[must_use]
    pub fn numperpage_or_default(&self) -> u32 {
        self.numperpage.unwrap_or(1_u32)
    }
    /// Field 6 , or its schema default when absent.
    #[must_use]
    pub fn r#type_or_default(&self) -> &str {
        self.r#type.as_deref().unwrap_or("myfiles")
    }
    /// Field 7 , or its schema default when absent.
    #[must_use]
    pub fn sortmethod_or_default(&self) -> &str {
        self.sortmethod.as_deref().unwrap_or("lastupdated")
    }
    /// Field 27 , or its schema default when absent.
    #[must_use]
    pub fn cache_max_age_seconds_or_default(&self) -> u32 {
        self.cache_max_age_seconds.unwrap_or(0_u32)
    }
    /// Field 29 , or its schema default when absent.
    #[must_use]
    pub fn language_or_default(&self) -> i32 {
        self.language.unwrap_or(0_i32)
    }
    /// Field 19 , or its schema default when absent.
    #[must_use]
    pub fn return_vote_data_or_default(&self) -> bool {
        self.return_vote_data.unwrap_or(true)
    }
    /// Field 21 , or its schema default when absent.
    #[must_use]
    pub fn return_kv_tags_or_default(&self) -> bool {
        self.return_kv_tags.unwrap_or(true)
    }
    /// Field 24 , or its schema default when absent.
    #[must_use]
    pub fn return_short_description_or_default(&self) -> bool {
        self.return_short_description.unwrap_or(true)
    }
    /// Field 28 , or its schema default when absent.
    #[must_use]
    pub fn return_metadata_or_default(&self) -> bool {
        self.return_metadata.unwrap_or(false)
    }
    /// Field 35 , or its schema default when absent.
    #[must_use]
    pub fn return_reactions_or_default(&self) -> bool {
        self.return_reactions.unwrap_or(false)
    }
    /// Field 33 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
}

impl Message for CPublishedFile_GetUserFiles_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.shortcutid = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.page = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.numperpage = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.r#type = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.sortmethod = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.privacy = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.requiredtags.push(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.excludedtags.push(decoder.read_string()?.to_owned());
                }
                30 => {
                    self.required_kv_tags.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::KVTag::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                14 => {
                    self.filetype = Some(decoder.read_varint()? as u32);
                }
                15 => {
                    self.creator_appid = Some(decoder.read_varint()? as u32);
                }
                16 => {
                    self.match_cloud_filename = Some(decoder.read_string()?.to_owned());
                }
                27 => {
                    self.cache_max_age_seconds = Some(decoder.read_varint()? as u32);
                }
                29 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                34 => {
                    self.taggroups.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::TagGroup::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                39 => {
                    self.date_range_created = Some({
                        let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::DateRange::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                40 => {
                    self.date_range_updated = Some({
                        let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_request::DateRange::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                37 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.excluded_content_descriptors,
                    |d: &mut Decoder<'_>| {
                        Ok(crate::enums_productinfo::EContentDescriptorID::from(
                            d.read_varint()? as i32,
                        ))
                    },
                )?,
                38 => {
                    self.admin_query = Some(decoder.read_bool()?);
                }
                17 => {
                    self.totalonly = Some(decoder.read_bool()?);
                }
                18 => {
                    self.ids_only = Some(decoder.read_bool()?);
                }
                19 => {
                    self.return_vote_data = Some(decoder.read_bool()?);
                }
                20 => {
                    self.return_tags = Some(decoder.read_bool()?);
                }
                21 => {
                    self.return_kv_tags = Some(decoder.read_bool()?);
                }
                22 => {
                    self.return_previews = Some(decoder.read_bool()?);
                }
                23 => {
                    self.return_children = Some(decoder.read_bool()?);
                }
                24 => {
                    self.return_short_description = Some(decoder.read_bool()?);
                }
                26 => {
                    self.return_for_sale_data = Some(decoder.read_bool()?);
                }
                28 => {
                    self.return_metadata = Some(decoder.read_bool()?);
                }
                31 => {
                    self.return_playtime_stats = Some(decoder.read_varint()? as u32);
                }
                32 => {
                    self.strip_description_bbcode = Some(decoder.read_bool()?);
                }
                35 => {
                    self.return_reactions = Some(decoder.read_bool()?);
                }
                25 => {
                    self.startindex_override = Some(decoder.read_varint()? as u32);
                }
                33 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                36 => {
                    self.return_apps = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.shortcutid {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.page {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.numperpage {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.r#type {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.sortmethod {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.privacy {
            encoder.write_varint_field(9, u64::from(*value));
        }
        for value in &self.requiredtags {
            encoder.write_string_field(10, value);
        }
        for value in &self.excludedtags {
            encoder.write_string_field(11, value);
        }
        for value in &self.required_kv_tags {
            encoder.write_message_field(30, value);
        }
        if let Some(value) = &self.filetype {
            encoder.write_varint_field(14, u64::from(*value));
        }
        if let Some(value) = &self.creator_appid {
            encoder.write_varint_field(15, u64::from(*value));
        }
        if let Some(value) = &self.match_cloud_filename {
            encoder.write_string_field(16, value);
        }
        if let Some(value) = &self.cache_max_age_seconds {
            encoder.write_varint_field(27, u64::from(*value));
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(29, *value);
        }
        for value in &self.taggroups {
            encoder.write_message_field(34, value);
        }
        if let Some(value) = &self.date_range_created {
            encoder.write_message_field(39, value);
        }
        if let Some(value) = &self.date_range_updated {
            encoder.write_message_field(40, value);
        }
        for value in &self.excluded_content_descriptors {
            encoder.write_varint_field(37, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.admin_query {
            encoder.write_bool_field(38, *value);
        }
        if let Some(value) = &self.totalonly {
            encoder.write_bool_field(17, *value);
        }
        if let Some(value) = &self.ids_only {
            encoder.write_bool_field(18, *value);
        }
        if let Some(value) = &self.return_vote_data {
            encoder.write_bool_field(19, *value);
        }
        if let Some(value) = &self.return_tags {
            encoder.write_bool_field(20, *value);
        }
        if let Some(value) = &self.return_kv_tags {
            encoder.write_bool_field(21, *value);
        }
        if let Some(value) = &self.return_previews {
            encoder.write_bool_field(22, *value);
        }
        if let Some(value) = &self.return_children {
            encoder.write_bool_field(23, *value);
        }
        if let Some(value) = &self.return_short_description {
            encoder.write_bool_field(24, *value);
        }
        if let Some(value) = &self.return_for_sale_data {
            encoder.write_bool_field(26, *value);
        }
        if let Some(value) = &self.return_metadata {
            encoder.write_bool_field(28, *value);
        }
        if let Some(value) = &self.return_playtime_stats {
            encoder.write_varint_field(31, u64::from(*value));
        }
        if let Some(value) = &self.strip_description_bbcode {
            encoder.write_bool_field(32, *value);
        }
        if let Some(value) = &self.return_reactions {
            encoder.write_bool_field(35, *value);
        }
        if let Some(value) = &self.startindex_override {
            encoder.write_varint_field(25, u64::from(*value));
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(33, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.return_apps {
            encoder.write_bool_field(36, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetUserFiles_Response`].
pub mod c_published_file_get_user_files_response {
    use super::*;

    /// `App` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct App {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub name: Option<String>,
        /// Field 3.
        pub shortcutid: Option<u32>,
        /// Field 4.
        pub private: Option<bool>,
    }

    impl Message for App {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.name = Some(decoder.read_string()?.to_owned());
                    }
                    3 => {
                        self.shortcutid = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.private = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.appid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.name {
                encoder.write_string_field(2, value);
            }
            if let Some(value) = &self.shortcutid {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.private {
                encoder.write_bool_field(4, *value);
            }
        }
    }
}

/// `CPublishedFile_GetUserFiles_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetUserFiles_Response {
    /// Field 1.
    pub total: Option<u32>,
    /// Field 2.
    pub startindex: Option<u32>,
    /// Field 3.
    pub publishedfiledetails: Vec<crate::steammessages_publishedfile_steamclient::PublishedFileDetails>,
    /// Field 4.
    pub apps: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_response::App>,
}

impl Message for CPublishedFile_GetUserFiles_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.total = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.startindex = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.publishedfiledetails.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileDetails::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                4 => {
                    self.apps.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_user_files_response::App::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.total {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.startindex {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.publishedfiledetails {
            encoder.write_message_field(3, value);
        }
        for value in &self.apps {
            encoder.write_message_field(4, value);
        }
    }
}

/// `CPublishedFile_AreFilesInSubscriptionList_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_AreFilesInSubscriptionList_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub publishedfileids: Vec<u64>,
    /// Field 3.
    pub listtype: Option<u32>,
    /// Field 4.
    pub filetype: Option<u32>,
    /// Field 5.
    pub workshopfiletype: Option<u32>,
}

impl Message for CPublishedFile_AreFilesInSubscriptionList_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.publishedfileids,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                3 => {
                    self.listtype = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.filetype = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.workshopfiletype = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.publishedfileids {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.listtype {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.filetype {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.workshopfiletype {
            encoder.write_varint_field(5, u64::from(*value));
        }
    }
}

/// Types nested inside [`CPublishedFile_AreFilesInSubscriptionList_Response`].
pub mod c_published_file_are_files_in_subscription_list_response {
    use super::*;

    /// `InList` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct InList {
        /// Field 1.
        pub publishedfileid: Option<u64>,
        /// Field 2.
        pub inlist: Option<bool>,
    }

    impl Message for InList {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.publishedfileid = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.inlist = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.publishedfileid {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.inlist {
                encoder.write_bool_field(2, *value);
            }
        }
    }
}

/// `CPublishedFile_AreFilesInSubscriptionList_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_AreFilesInSubscriptionList_Response {
    /// Field 1.
    pub files: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_are_files_in_subscription_list_response::InList>,
}

impl Message for CPublishedFile_AreFilesInSubscriptionList_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.files.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_are_files_in_subscription_list_response::InList::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.files {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CPublishedFile_Update_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Update_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub publishedfileid: Option<u64>,
    /// Field 3.
    pub title: Option<String>,
    /// Field 4.
    pub file_description: Option<String>,
    /// Field 5.
    pub visibility: Option<u32>,
    /// Field 6.
    pub tags: Vec<String>,
    /// Field 7.
    pub filename: Option<String>,
    /// Field 8.
    pub preview_filename: Option<String>,
    /// Field 10.
    pub spoiler_tag: Option<bool>,
    /// Field 15.
    pub image_width: Option<u32>,
    /// Field 16.
    pub image_height: Option<u32>,
    /// Field 17.
    pub language: Option<i32>,
}

impl Message for CPublishedFile_Update_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.publishedfileid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.title = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.file_description = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.visibility = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.tags.push(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.filename = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.preview_filename = Some(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.spoiler_tag = Some(decoder.read_bool()?);
                }
                15 => {
                    self.image_width = Some(decoder.read_varint()? as u32);
                }
                16 => {
                    self.image_height = Some(decoder.read_varint()? as u32);
                }
                17 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.publishedfileid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.title {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.file_description {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.visibility {
            encoder.write_varint_field(5, u64::from(*value));
        }
        for value in &self.tags {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.filename {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.preview_filename {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.spoiler_tag {
            encoder.write_bool_field(10, *value);
        }
        if let Some(value) = &self.image_width {
            encoder.write_varint_field(15, u64::from(*value));
        }
        if let Some(value) = &self.image_height {
            encoder.write_varint_field(16, u64::from(*value));
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(17, *value);
        }
    }
}

/// `CPublishedFile_Update_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Update_Response {}

impl Message for CPublishedFile_Update_Response {
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

/// `CPublishedFile_Delete_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Delete_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 5.
    pub appid: Option<u32>,
}

impl Message for CPublishedFile_Delete_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_fixed64()?);
                }
                5 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(5, u64::from(*value));
        }
    }
}

/// `CPublishedFile_Delete_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_Delete_Response {}

impl Message for CPublishedFile_Delete_Response {
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

/// `CPublishedFile_GetChangeHistoryEntry_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetChangeHistoryEntry_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub timestamp: Option<u32>,
    /// Field 3.
    pub language: Option<i32>,
}

impl Message for CPublishedFile_GetChangeHistoryEntry_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.timestamp = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.timestamp {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(3, *value);
        }
    }
}

/// `CPublishedFile_GetChangeHistoryEntry_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetChangeHistoryEntry_Response {
    /// Field 1.
    pub change_description: Option<String>,
    /// Field 2.
    pub language: Option<i32>,
    /// Field 3.
    pub saved_snapshot: Option<bool>,
    /// Field 4.
    pub snapshot_game_branch_min: Option<String>,
    /// Field 5.
    pub snapshot_game_branch_max: Option<String>,
    /// Field 6.
    pub manifest_id: Option<u64>,
    /// Field 7.
    pub accountid: Option<u32>,
}

impl Message for CPublishedFile_GetChangeHistoryEntry_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.change_description = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.saved_snapshot = Some(decoder.read_bool()?);
                }
                4 => {
                    self.snapshot_game_branch_min = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.snapshot_game_branch_max = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.manifest_id = Some(decoder.read_fixed64()?);
                }
                7 => {
                    self.accountid = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.change_description {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.saved_snapshot {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.snapshot_game_branch_min {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.snapshot_game_branch_max {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.manifest_id {
            encoder.write_fixed64_field(6, *value);
        }
        if let Some(value) = &self.accountid {
            encoder.write_varint_field(7, u64::from(*value));
        }
    }
}

/// `CPublishedFile_GetChangeHistory_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetChangeHistory_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub total_only: Option<bool>,
    /// Field 3.
    pub startindex: Option<u32>,
    /// Field 4.
    pub count: Option<u32>,
    /// Field 5.
    pub language: Option<i32>,
}

impl CPublishedFile_GetChangeHistory_Request {
    /// Field 5 , or its schema default when absent.
    #[must_use]
    pub fn language_or_default(&self) -> i32 {
        self.language.unwrap_or(0_i32)
    }
}

impl Message for CPublishedFile_GetChangeHistory_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.total_only = Some(decoder.read_bool()?);
                }
                3 => {
                    self.startindex = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.count = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.total_only {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.startindex {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.count {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(5, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetChangeHistory_Response`].
pub mod c_published_file_get_change_history_response {
    use super::*;

    /// `ChangeLog` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ChangeLog {
        /// Field 1.
        pub timestamp: Option<u32>,
        /// Field 2.
        pub change_description: Option<String>,
        /// Field 3.
        pub language: Option<i32>,
        /// Field 4.
        pub saved_snapshot: Option<bool>,
        /// Field 5.
        pub snapshot_game_branch_min: Option<String>,
        /// Field 6.
        pub snapshot_game_branch_max: Option<String>,
        /// Field 7.
        pub manifest_id: Option<u64>,
        /// Field 8.
        pub accountid: Option<u32>,
    }

    impl Message for ChangeLog {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.timestamp = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.change_description = Some(decoder.read_string()?.to_owned());
                    }
                    3 => {
                        self.language = Some(decoder.read_varint()? as i32);
                    }
                    4 => {
                        self.saved_snapshot = Some(decoder.read_bool()?);
                    }
                    5 => {
                        self.snapshot_game_branch_min = Some(decoder.read_string()?.to_owned());
                    }
                    6 => {
                        self.snapshot_game_branch_max = Some(decoder.read_string()?.to_owned());
                    }
                    7 => {
                        self.manifest_id = Some(decoder.read_fixed64()?);
                    }
                    8 => {
                        self.accountid = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.timestamp {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.change_description {
                encoder.write_string_field(2, value);
            }
            if let Some(value) = &self.language {
                encoder.write_int32_field(3, *value);
            }
            if let Some(value) = &self.saved_snapshot {
                encoder.write_bool_field(4, *value);
            }
            if let Some(value) = &self.snapshot_game_branch_min {
                encoder.write_string_field(5, value);
            }
            if let Some(value) = &self.snapshot_game_branch_max {
                encoder.write_string_field(6, value);
            }
            if let Some(value) = &self.manifest_id {
                encoder.write_fixed64_field(7, *value);
            }
            if let Some(value) = &self.accountid {
                encoder.write_varint_field(8, u64::from(*value));
            }
        }
    }
}

/// `CPublishedFile_GetChangeHistory_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetChangeHistory_Response {
    /// Field 1.
    pub changes: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_change_history_response::ChangeLog>,
    /// Field 2.
    pub total: Option<u32>,
}

impl Message for CPublishedFile_GetChangeHistory_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.changes.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_change_history_response::ChangeLog::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.total = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.changes {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.total {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CPublishedFile_RefreshVotingQueue_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_RefreshVotingQueue_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub matching_file_type: Option<u32>,
    /// Field 3.
    pub tags: Vec<String>,
    /// Field 4.
    pub match_all_tags: Option<bool>,
    /// Field 5.
    pub excluded_tags: Vec<String>,
    /// Field 6.
    pub desired_queue_size: Option<u32>,
    /// Field 8.
    pub desired_revision:
        Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
}

impl CPublishedFile_RefreshVotingQueue_Request {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn match_all_tags_or_default(&self) -> bool {
        self.match_all_tags.unwrap_or(true)
    }
    /// Field 8 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
}

impl Message for CPublishedFile_RefreshVotingQueue_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.matching_file_type = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.tags.push(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.match_all_tags = Some(decoder.read_bool()?);
                }
                5 => {
                    self.excluded_tags.push(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.desired_queue_size = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.matching_file_type {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.tags {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.match_all_tags {
            encoder.write_bool_field(4, *value);
        }
        for value in &self.excluded_tags {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.desired_queue_size {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(8, i64::from(value.value()) as u64);
        }
    }
}

/// `CPublishedFile_RefreshVotingQueue_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_RefreshVotingQueue_Response {}

impl Message for CPublishedFile_RefreshVotingQueue_Response {
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

/// Types nested inside [`CPublishedFile_QueryFiles_Request`].
pub mod c_published_file_query_files_request {
    use super::*;

    /// `KVTag` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct KVTag {
        /// Field 1.
        pub key: Option<String>,
        /// Field 2.
        pub value: Option<String>,
    }

    impl Message for KVTag {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.key = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.value = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.key {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.value {
                encoder.write_string_field(2, value);
            }
        }
    }

    /// `TagGroup` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct TagGroup {
        /// Field 1.
        pub tags: Vec<String>,
    }

    impl Message for TagGroup {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.tags.push(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            for value in &self.tags {
                encoder.write_string_field(1, value);
            }
        }
    }

    /// `DateRange` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct DateRange {
        /// Field 1.
        pub timestamp_start: Option<u32>,
        /// Field 2.
        pub timestamp_end: Option<u32>,
    }

    impl Message for DateRange {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.timestamp_start = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.timestamp_end = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.timestamp_start {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.timestamp_end {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CPublishedFile_QueryFiles_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_QueryFiles_Request {
    /// Field 1.
    pub query_type: Option<u32>,
    /// Field 2.
    pub page: Option<u32>,
    /// Field 39.
    pub cursor: Option<String>,
    /// Field 3.
    pub numperpage: Option<u32>,
    /// Field 4.
    pub creator_appid: Option<u32>,
    /// Field 5.
    pub appid: Option<u32>,
    /// Field 6.
    pub requiredtags: Vec<String>,
    /// Field 7.
    pub excludedtags: Vec<String>,
    /// Field 8.
    pub match_all_tags: Option<bool>,
    /// Field 9.
    pub required_flags: Vec<String>,
    /// Field 10.
    pub omitted_flags: Vec<String>,
    /// Field 11.
    pub search_text: Option<String>,
    /// Field 12.
    pub filetype: Option<u32>,
    /// Field 13.
    pub child_publishedfileid: Option<u64>,
    /// Field 14.
    pub days: Option<u32>,
    /// Field 15.
    pub include_recent_votes_only: Option<bool>,
    /// Field 31.
    pub cache_max_age_seconds: Option<u32>,
    /// Field 33.
    pub language: Option<i32>,
    /// Field 34.
    pub required_kv_tags: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::KVTag>,
    /// Field 42.
    pub taggroups: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::TagGroup>,
    /// Field 44.
    pub date_range_created: Option<crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::DateRange>,
    /// Field 45.
    pub date_range_updated: Option<crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::DateRange>,
    /// Field 46.
    pub excluded_content_descriptors: Vec<crate::enums_productinfo::EContentDescriptorID>,
    /// Field 47.
    pub admin_query: Option<bool>,
    /// Field 48.
    pub special_filter: Option<crate::steammessages_publishedfile_steamclient::EQueryFilesSpecialFilter>,
    /// Field 49.
    pub appids_required_for_use: Vec<u32>,
    /// Field 51.
    pub excluded_appids_required_for_use: Vec<u32>,
    /// Field 50.
    pub search_text_target: Option<crate::steammessages_publishedfile_steamclient::EQueryFilesSearchTextTarget>,
    /// Field 16.
    pub totalonly: Option<bool>,
    /// Field 35.
    pub ids_only: Option<bool>,
    /// Field 17.
    pub return_vote_data: Option<bool>,
    /// Field 18.
    pub return_tags: Option<bool>,
    /// Field 19.
    pub return_kv_tags: Option<bool>,
    /// Field 20.
    pub return_previews: Option<bool>,
    /// Field 21.
    pub return_children: Option<bool>,
    /// Field 22.
    pub return_short_description: Option<bool>,
    /// Field 30.
    pub return_for_sale_data: Option<bool>,
    /// Field 32.
    pub return_metadata: Option<bool>,
    /// Field 36.
    pub return_playtime_stats: Option<u32>,
    /// Field 37.
    pub return_details: Option<bool>,
    /// Field 38.
    pub strip_description_bbcode: Option<bool>,
    /// Field 40.
    pub desired_revision: Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 43.
    pub return_reactions: Option<bool>,
}

impl CPublishedFile_QueryFiles_Request {
    /// Field 3 , or its schema default when absent.
    #[must_use]
    pub fn numperpage_or_default(&self) -> u32 {
        self.numperpage.unwrap_or(1_u32)
    }
    /// Field 8 , or its schema default when absent.
    #[must_use]
    pub fn match_all_tags_or_default(&self) -> bool {
        self.match_all_tags.unwrap_or(true)
    }
    /// Field 31 , or its schema default when absent.
    #[must_use]
    pub fn cache_max_age_seconds_or_default(&self) -> u32 {
        self.cache_max_age_seconds.unwrap_or(0_u32)
    }
    /// Field 33 , or its schema default when absent.
    #[must_use]
    pub fn language_or_default(&self) -> i32 {
        self.language.unwrap_or(0_i32)
    }
    /// Field 48 , or its schema default when absent.
    #[must_use]
    pub fn special_filter_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EQueryFilesSpecialFilter {
        self.special_filter.unwrap_or(crate::steammessages_publishedfile_steamclient::EQueryFilesSpecialFilter::k_EQueryFilesSpecialFilter_None)
    }
    /// Field 50 , or its schema default when absent.
    #[must_use]
    pub fn search_text_target_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EQueryFilesSearchTextTarget {
        self.search_text_target.unwrap_or(crate::steammessages_publishedfile_steamclient::EQueryFilesSearchTextTarget::k_EQueryFilesSearchTextTarget_AllText)
    }
    /// Field 32 , or its schema default when absent.
    #[must_use]
    pub fn return_metadata_or_default(&self) -> bool {
        self.return_metadata.unwrap_or(false)
    }
    /// Field 40 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
    /// Field 43 , or its schema default when absent.
    #[must_use]
    pub fn return_reactions_or_default(&self) -> bool {
        self.return_reactions.unwrap_or(false)
    }
}

impl Message for CPublishedFile_QueryFiles_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.query_type = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.page = Some(decoder.read_varint()? as u32);
                }
                39 => {
                    self.cursor = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.numperpage = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.creator_appid = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.requiredtags.push(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.excludedtags.push(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.match_all_tags = Some(decoder.read_bool()?);
                }
                9 => {
                    self.required_flags.push(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.omitted_flags.push(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.search_text = Some(decoder.read_string()?.to_owned());
                }
                12 => {
                    self.filetype = Some(decoder.read_varint()? as u32);
                }
                13 => {
                    self.child_publishedfileid = Some(decoder.read_fixed64()?);
                }
                14 => {
                    self.days = Some(decoder.read_varint()? as u32);
                }
                15 => {
                    self.include_recent_votes_only = Some(decoder.read_bool()?);
                }
                31 => {
                    self.cache_max_age_seconds = Some(decoder.read_varint()? as u32);
                }
                33 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                34 => {
                    self.required_kv_tags.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::KVTag::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                42 => {
                    self.taggroups.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::TagGroup::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                44 => {
                    self.date_range_created = Some({
                        let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::DateRange::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                45 => {
                    self.date_range_updated = Some({
                        let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_query_files_request::DateRange::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                46 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.excluded_content_descriptors,
                    |d: &mut Decoder<'_>| {
                        Ok(crate::enums_productinfo::EContentDescriptorID::from(
                            d.read_varint()? as i32,
                        ))
                    },
                )?,
                47 => {
                    self.admin_query = Some(decoder.read_bool()?);
                }
                48 => {
                    self.special_filter = Some(crate::steammessages_publishedfile_steamclient::EQueryFilesSpecialFilter::from(decoder.read_varint()? as i32));
                }
                49 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.appids_required_for_use,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                51 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.excluded_appids_required_for_use,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                50 => {
                    self.search_text_target = Some(crate::steammessages_publishedfile_steamclient::EQueryFilesSearchTextTarget::from(decoder.read_varint()? as i32));
                }
                16 => {
                    self.totalonly = Some(decoder.read_bool()?);
                }
                35 => {
                    self.ids_only = Some(decoder.read_bool()?);
                }
                17 => {
                    self.return_vote_data = Some(decoder.read_bool()?);
                }
                18 => {
                    self.return_tags = Some(decoder.read_bool()?);
                }
                19 => {
                    self.return_kv_tags = Some(decoder.read_bool()?);
                }
                20 => {
                    self.return_previews = Some(decoder.read_bool()?);
                }
                21 => {
                    self.return_children = Some(decoder.read_bool()?);
                }
                22 => {
                    self.return_short_description = Some(decoder.read_bool()?);
                }
                30 => {
                    self.return_for_sale_data = Some(decoder.read_bool()?);
                }
                32 => {
                    self.return_metadata = Some(decoder.read_bool()?);
                }
                36 => {
                    self.return_playtime_stats = Some(decoder.read_varint()? as u32);
                }
                37 => {
                    self.return_details = Some(decoder.read_bool()?);
                }
                38 => {
                    self.strip_description_bbcode = Some(decoder.read_bool()?);
                }
                40 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                43 => {
                    self.return_reactions = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.query_type {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.page {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.cursor {
            encoder.write_string_field(39, value);
        }
        if let Some(value) = &self.numperpage {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.creator_appid {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(5, u64::from(*value));
        }
        for value in &self.requiredtags {
            encoder.write_string_field(6, value);
        }
        for value in &self.excludedtags {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.match_all_tags {
            encoder.write_bool_field(8, *value);
        }
        for value in &self.required_flags {
            encoder.write_string_field(9, value);
        }
        for value in &self.omitted_flags {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.search_text {
            encoder.write_string_field(11, value);
        }
        if let Some(value) = &self.filetype {
            encoder.write_varint_field(12, u64::from(*value));
        }
        if let Some(value) = &self.child_publishedfileid {
            encoder.write_fixed64_field(13, *value);
        }
        if let Some(value) = &self.days {
            encoder.write_varint_field(14, u64::from(*value));
        }
        if let Some(value) = &self.include_recent_votes_only {
            encoder.write_bool_field(15, *value);
        }
        if let Some(value) = &self.cache_max_age_seconds {
            encoder.write_varint_field(31, u64::from(*value));
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(33, *value);
        }
        for value in &self.required_kv_tags {
            encoder.write_message_field(34, value);
        }
        for value in &self.taggroups {
            encoder.write_message_field(42, value);
        }
        if let Some(value) = &self.date_range_created {
            encoder.write_message_field(44, value);
        }
        if let Some(value) = &self.date_range_updated {
            encoder.write_message_field(45, value);
        }
        for value in &self.excluded_content_descriptors {
            encoder.write_varint_field(46, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.admin_query {
            encoder.write_bool_field(47, *value);
        }
        if let Some(value) = &self.special_filter {
            encoder.write_varint_field(48, i64::from(value.value()) as u64);
        }
        for value in &self.appids_required_for_use {
            encoder.write_varint_field(49, u64::from(*value));
        }
        for value in &self.excluded_appids_required_for_use {
            encoder.write_varint_field(51, u64::from(*value));
        }
        if let Some(value) = &self.search_text_target {
            encoder.write_varint_field(50, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.totalonly {
            encoder.write_bool_field(16, *value);
        }
        if let Some(value) = &self.ids_only {
            encoder.write_bool_field(35, *value);
        }
        if let Some(value) = &self.return_vote_data {
            encoder.write_bool_field(17, *value);
        }
        if let Some(value) = &self.return_tags {
            encoder.write_bool_field(18, *value);
        }
        if let Some(value) = &self.return_kv_tags {
            encoder.write_bool_field(19, *value);
        }
        if let Some(value) = &self.return_previews {
            encoder.write_bool_field(20, *value);
        }
        if let Some(value) = &self.return_children {
            encoder.write_bool_field(21, *value);
        }
        if let Some(value) = &self.return_short_description {
            encoder.write_bool_field(22, *value);
        }
        if let Some(value) = &self.return_for_sale_data {
            encoder.write_bool_field(30, *value);
        }
        if let Some(value) = &self.return_metadata {
            encoder.write_bool_field(32, *value);
        }
        if let Some(value) = &self.return_playtime_stats {
            encoder.write_varint_field(36, u64::from(*value));
        }
        if let Some(value) = &self.return_details {
            encoder.write_bool_field(37, *value);
        }
        if let Some(value) = &self.strip_description_bbcode {
            encoder.write_bool_field(38, *value);
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(40, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.return_reactions {
            encoder.write_bool_field(43, *value);
        }
    }
}

/// `CPublishedFile_QueryFiles_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_QueryFiles_Response {
    /// Field 1.
    pub total: Option<u32>,
    /// Field 2.
    pub publishedfiledetails:
        Vec<crate::steammessages_publishedfile_steamclient::PublishedFileDetails>,
    /// Field 3.
    pub next_cursor: Option<String>,
}

impl Message for CPublishedFile_QueryFiles_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.total = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.publishedfiledetails.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileDetails::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                3 => {
                    self.next_cursor = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.total {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.publishedfiledetails {
            encoder.write_message_field(2, value);
        }
        if let Some(value) = &self.next_cursor {
            encoder.write_string_field(3, value);
        }
    }
}

/// `CPublishedFile_AddAppRelationship_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_AddAppRelationship_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub appid: Option<u32>,
    /// Field 3.
    pub relationship: Option<u32>,
}

impl Message for CPublishedFile_AddAppRelationship_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.relationship = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.relationship {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CPublishedFile_AddAppRelationship_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_AddAppRelationship_Response {}

impl Message for CPublishedFile_AddAppRelationship_Response {
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

/// `CPublishedFile_RemoveAppRelationship_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_RemoveAppRelationship_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub appid: Option<u32>,
    /// Field 3.
    pub relationship: Option<u32>,
}

impl Message for CPublishedFile_RemoveAppRelationship_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.relationship = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.relationship {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CPublishedFile_RemoveAppRelationship_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_RemoveAppRelationship_Response {}

impl Message for CPublishedFile_RemoveAppRelationship_Response {
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

/// `CPublishedFile_GetAppRelationships_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetAppRelationships_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
}

impl Message for CPublishedFile_GetAppRelationships_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetAppRelationships_Response`].
pub mod c_published_file_get_app_relationships_response {
    use super::*;

    /// `AppRelationship` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AppRelationship {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub relationship: Option<u32>,
    }

    impl Message for AppRelationship {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.relationship = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.appid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.relationship {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CPublishedFile_GetAppRelationships_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetAppRelationships_Response {
    /// Field 3.
    pub app_relationships: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_app_relationships_response::AppRelationship>,
}

impl Message for CPublishedFile_GetAppRelationships_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                3 => {
                    self.app_relationships.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_app_relationships_response::AppRelationship::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.app_relationships {
            encoder.write_message_field(3, value);
        }
    }
}

/// `CPublishedFile_GetAppRelationshipsBatched_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetAppRelationshipsBatched_Request {
    /// Field 1.
    pub publishedfileids: Vec<u64>,
    /// Field 2.
    pub filter_relationship: Option<u32>,
}

impl Message for CPublishedFile_GetAppRelationshipsBatched_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.publishedfileids,
                    |d: &mut Decoder<'_>| d.read_varint(),
                )?,
                2 => {
                    self.filter_relationship = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.publishedfileids {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.filter_relationship {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CPublishedFile_GetAppRelationshipsBatched_Response`].
pub mod c_published_file_get_app_relationships_batched_response {
    use super::*;

    /// `AppRelationship` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AppRelationship {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub relationship: Option<u32>,
    }

    impl Message for AppRelationship {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.relationship = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.appid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.relationship {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }

    /// `PublishedFileAppRelationship` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PublishedFileAppRelationship {
        /// Field 1.
        pub publishedfileid: Option<u64>,
        /// Field 2.
        pub result: Option<u32>,
        /// Field 3.
        pub app_relationships: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_app_relationships_batched_response::AppRelationship>,
    }

    impl Message for PublishedFileAppRelationship {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.publishedfileid = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.result = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.app_relationships.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_app_relationships_batched_response::AppRelationship::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.publishedfileid {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.result {
                encoder.write_varint_field(2, u64::from(*value));
            }
            for value in &self.app_relationships {
                encoder.write_message_field(3, value);
            }
        }
    }
}

/// `CPublishedFile_GetAppRelationshipsBatched_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetAppRelationshipsBatched_Response {
    /// Field 1.
    pub relationships: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_app_relationships_batched_response::PublishedFileAppRelationship>,
}

impl Message for CPublishedFile_GetAppRelationshipsBatched_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.relationships.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_app_relationships_batched_response::PublishedFileAppRelationship::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.relationships {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CPublishedFile_StartPlaytimeTracking_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_StartPlaytimeTracking_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub publishedfileids: Vec<u64>,
}

impl Message for CPublishedFile_StartPlaytimeTracking_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.publishedfileids,
                    |d: &mut Decoder<'_>| d.read_varint(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.publishedfileids {
            encoder.write_varint_field(2, *value);
        }
    }
}

/// `CPublishedFile_StartPlaytimeTracking_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_StartPlaytimeTracking_Response {}

impl Message for CPublishedFile_StartPlaytimeTracking_Response {
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

/// `CPublishedFile_StopPlaytimeTracking_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_StopPlaytimeTracking_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub publishedfileids: Vec<u64>,
}

impl Message for CPublishedFile_StopPlaytimeTracking_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.publishedfileids,
                    |d: &mut Decoder<'_>| d.read_varint(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.publishedfileids {
            encoder.write_varint_field(2, *value);
        }
    }
}

/// `CPublishedFile_StopPlaytimeTracking_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_StopPlaytimeTracking_Response {}

impl Message for CPublishedFile_StopPlaytimeTracking_Response {
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

/// `CPublishedFile_StopPlaytimeTrackingForAllAppItems_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_StopPlaytimeTrackingForAllAppItems_Request {
    /// Field 1.
    pub appid: Option<u32>,
}

impl Message for CPublishedFile_StopPlaytimeTrackingForAllAppItems_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
    }
}

/// `CPublishedFile_StopPlaytimeTrackingForAllAppItems_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_StopPlaytimeTrackingForAllAppItems_Response {}

impl Message for CPublishedFile_StopPlaytimeTrackingForAllAppItems_Response {
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

/// Types nested inside [`CPublishedFile_SetPlaytimeForControllerConfigs_Request`].
pub mod c_published_file_set_playtime_for_controller_configs_request {
    use super::*;

    /// `ControllerConfigUsage` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ControllerConfigUsage {
        /// Field 1.
        pub publishedfileid: Option<u64>,
        /// Field 2.
        pub seconds_active: Option<f32>,
    }

    impl Message for ControllerConfigUsage {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.publishedfileid = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.seconds_active = Some(decoder.read_float()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.publishedfileid {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.seconds_active {
                encoder.write_float_field(2, *value);
            }
        }
    }
}

/// `CPublishedFile_SetPlaytimeForControllerConfigs_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_SetPlaytimeForControllerConfigs_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub controller_config_usage: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_set_playtime_for_controller_configs_request::ControllerConfigUsage>,
}

impl Message for CPublishedFile_SetPlaytimeForControllerConfigs_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.controller_config_usage.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_set_playtime_for_controller_configs_request::ControllerConfigUsage::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.controller_config_usage {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CPublishedFile_SetPlaytimeForControllerConfigs_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_SetPlaytimeForControllerConfigs_Response {}

impl Message for CPublishedFile_SetPlaytimeForControllerConfigs_Response {
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

/// `CPublishedFile_AddChild_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_AddChild_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub child_publishedfileid: Option<u64>,
}

impl Message for CPublishedFile_AddChild_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.child_publishedfileid = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.child_publishedfileid {
            encoder.write_varint_field(2, *value);
        }
    }
}

/// `CPublishedFile_AddChild_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_AddChild_Response {}

impl Message for CPublishedFile_AddChild_Response {
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

/// `CPublishedFile_RemoveChild_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_RemoveChild_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub child_publishedfileid: Option<u64>,
}

impl Message for CPublishedFile_RemoveChild_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.child_publishedfileid = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.child_publishedfileid {
            encoder.write_varint_field(2, *value);
        }
    }
}

/// `CPublishedFile_RemoveChild_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_RemoveChild_Response {}

impl Message for CPublishedFile_RemoveChild_Response {
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

/// `CPublishedFile_SetCollectionChildren_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_SetCollectionChildren_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub publishedfileid: Option<u64>,
    /// Field 3.
    pub children: Vec<u64>,
}

impl Message for CPublishedFile_SetCollectionChildren_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.children,
                    |d: &mut Decoder<'_>| d.read_varint(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(2, *value);
        }
        for value in &self.children {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CPublishedFile_SetCollectionChildren_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_SetCollectionChildren_Response {}

impl Message for CPublishedFile_SetCollectionChildren_Response {
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

/// `CPublishedFile_SetSubscriptionListFromCollection_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_SetSubscriptionListFromCollection_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub list_type: Option<u32>,
    /// Field 3.
    pub publishedfileid: Option<u64>,
    /// Field 4.
    pub add_only: Option<bool>,
}

impl Message for CPublishedFile_SetSubscriptionListFromCollection_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.list_type = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.publishedfileid = Some(decoder.read_varint()?);
                }
                4 => {
                    self.add_only = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.list_type {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.publishedfileid {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.add_only {
            encoder.write_bool_field(4, *value);
        }
    }
}

/// `CPublishedFile_SetSubscriptionListFromCollection_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_SetSubscriptionListFromCollection_Response {}

impl Message for CPublishedFile_SetSubscriptionListFromCollection_Response {
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

/// `CPublishedFile_GetUserVoteSummary_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetUserVoteSummary_Request {
    /// Field 1.
    pub publishedfileids: Vec<u64>,
}

impl Message for CPublishedFile_GetUserVoteSummary_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.publishedfileids,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.publishedfileids {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetUserVoteSummary_Response`].
pub mod c_published_file_get_user_vote_summary_response {
    use super::*;

    /// `VoteSummary` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct VoteSummary {
        /// Field 1.
        pub publishedfileid: Option<u64>,
        /// Field 2.
        pub vote_for: Option<bool>,
        /// Field 3.
        pub vote_against: Option<bool>,
        /// Field 4.
        pub reported: Option<bool>,
    }

    impl Message for VoteSummary {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.publishedfileid = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.vote_for = Some(decoder.read_bool()?);
                    }
                    3 => {
                        self.vote_against = Some(decoder.read_bool()?);
                    }
                    4 => {
                        self.reported = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.publishedfileid {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.vote_for {
                encoder.write_bool_field(2, *value);
            }
            if let Some(value) = &self.vote_against {
                encoder.write_bool_field(3, *value);
            }
            if let Some(value) = &self.reported {
                encoder.write_bool_field(4, *value);
            }
        }
    }
}

/// `CPublishedFile_GetUserVoteSummary_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetUserVoteSummary_Response {
    /// Field 1.
    pub summaries: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_user_vote_summary_response::VoteSummary>,
}

impl Message for CPublishedFile_GetUserVoteSummary_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.summaries.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_user_vote_summary_response::VoteSummary::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.summaries {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CPublishedFile_GetItemChanges_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetItemChanges_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub last_time_updated: Option<u32>,
    /// Field 3.
    pub num_items_max: Option<u32>,
    /// Field 4.
    pub desired_revision:
        Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 5.
    pub include_legacy_items: Option<bool>,
}

impl CPublishedFile_GetItemChanges_Request {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn desired_revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.desired_revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
}

impl Message for CPublishedFile_GetItemChanges_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.last_time_updated = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.num_items_max = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.desired_revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                5 => {
                    self.include_legacy_items = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.appid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.last_time_updated {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.num_items_max {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.desired_revision {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.include_legacy_items {
            encoder.write_bool_field(5, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetItemChanges_Response`].
pub mod c_published_file_get_item_changes_response {
    use super::*;

    /// `WorkshopItemInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct WorkshopItemInfo {
        /// Field 1.
        pub published_file_id: Option<u64>,
        /// Field 2.
        pub time_updated: Option<u32>,
        /// Field 3.
        pub manifest_id: Option<u64>,
        /// Field 4.
        pub author_snapshots:
            Vec<crate::steammessages_publishedfile_steamclient::PublishedFileAuthorSnapshot>,
        /// Field 5.
        pub flags: Option<u32>,
    }

    impl Message for WorkshopItemInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.time_updated = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.manifest_id = Some(decoder.read_fixed64()?);
                    }
                    4 => {
                        self.author_snapshots.push({ let mut nested = crate::steammessages_publishedfile_steamclient::PublishedFileAuthorSnapshot::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    5 => {
                        self.flags = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.published_file_id {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.time_updated {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.manifest_id {
                encoder.write_fixed64_field(3, *value);
            }
            for value in &self.author_snapshots {
                encoder.write_message_field(4, value);
            }
            if let Some(value) = &self.flags {
                encoder.write_varint_field(5, u64::from(*value));
            }
        }
    }
}

/// `CPublishedFile_GetItemChanges_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetItemChanges_Response {
    /// Field 1.
    pub update_time: Option<u32>,
    /// Field 2.
    pub workshop_items: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_item_changes_response::WorkshopItemInfo>,
}

impl Message for CPublishedFile_GetItemChanges_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.update_time = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.workshop_items.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_item_changes_response::WorkshopItemInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.update_time {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.workshop_items {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CPublishedFile_GetContentDescriptors_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetContentDescriptors_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
}

impl Message for CPublishedFile_GetContentDescriptors_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// Types nested inside [`CPublishedFile_GetContentDescriptors_Response`].
pub mod c_published_file_get_content_descriptors_response {
    use super::*;

    /// `ContentDescriptor` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ContentDescriptor {
        /// Field 1.
        pub descriptorid: Option<crate::enums_productinfo::EContentDescriptorID>,
        /// Field 2.
        pub accountid: Option<u32>,
        /// Field 3.
        pub timestamp: Option<u32>,
        /// Field 4.
        pub moderator_set: Option<bool>,
    }

    impl ContentDescriptor {
        /// Field 1 , or its schema default when absent.
        #[must_use]
        pub fn descriptorid_or_default(&self) -> crate::enums_productinfo::EContentDescriptorID {
            self.descriptorid.unwrap_or(crate::enums_productinfo::EContentDescriptorID::k_EContentDescriptor_NudityOrSexualContent)
        }
    }

    impl Message for ContentDescriptor {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.descriptorid =
                            Some(crate::enums_productinfo::EContentDescriptorID::from(
                                decoder.read_varint()? as i32,
                            ));
                    }
                    2 => {
                        self.accountid = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.timestamp = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.moderator_set = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.descriptorid {
                encoder.write_varint_field(1, i64::from(value.value()) as u64);
            }
            if let Some(value) = &self.accountid {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.timestamp {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.moderator_set {
                encoder.write_bool_field(4, *value);
            }
        }
    }
}

/// `CPublishedFile_GetContentDescriptors_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_GetContentDescriptors_Response {
    /// Field 1.
    pub content_descriptors: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_get_content_descriptors_response::ContentDescriptor>,
}

impl Message for CPublishedFile_GetContentDescriptors_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.content_descriptors.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_get_content_descriptors_response::ContentDescriptor::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.content_descriptors {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CPublishedFile_UpdateContentDescriptors_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_UpdateContentDescriptors_Request {
    /// Field 1.
    pub publishedfileid: Option<u64>,
    /// Field 2.
    pub descriptors_to_add: Vec<crate::enums_productinfo::EContentDescriptorID>,
    /// Field 3.
    pub descriptors_to_remove: Vec<crate::enums_productinfo::EContentDescriptorID>,
}

impl Message for CPublishedFile_UpdateContentDescriptors_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publishedfileid = Some(decoder.read_fixed64()?);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.descriptors_to_add,
                    |d: &mut Decoder<'_>| {
                        Ok(crate::enums_productinfo::EContentDescriptorID::from(
                            d.read_varint()? as i32,
                        ))
                    },
                )?,
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.descriptors_to_remove,
                    |d: &mut Decoder<'_>| {
                        Ok(crate::enums_productinfo::EContentDescriptorID::from(
                            d.read_varint()? as i32,
                        ))
                    },
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publishedfileid {
            encoder.write_fixed64_field(1, *value);
        }
        for value in &self.descriptors_to_add {
            encoder.write_varint_field(2, i64::from(value.value()) as u64);
        }
        for value in &self.descriptors_to_remove {
            encoder.write_varint_field(3, i64::from(value.value()) as u64);
        }
    }
}

/// `CPublishedFile_UpdateContentDescriptors_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_UpdateContentDescriptors_Response {
    /// Field 1.
    pub timestamp_updated: Option<u32>,
}

impl Message for CPublishedFile_UpdateContentDescriptors_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.timestamp_updated = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.timestamp_updated {
            encoder.write_varint_field(1, u64::from(*value));
        }
    }
}

/// Types nested inside [`CPublishedFile_FileSubscribed_Notification`].
pub mod c_published_file_file_subscribed_notification {
    use super::*;

    /// `RevisionData` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct RevisionData {
        /// Field 1.
        pub revision:
            Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
        /// Field 2.
        pub file_hcontent: Option<u64>,
        /// Field 3.
        pub rtime_updated: Option<u32>,
        /// Field 4.
        pub game_branch_min: Option<String>,
        /// Field 5.
        pub game_branch_max: Option<String>,
    }

    impl RevisionData {
        /// Field 1 , or its schema default when absent.
        #[must_use]
        pub fn revision_or_default(
            &self,
        ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
            self.revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
        }
    }

    impl Message for RevisionData {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                    }
                    2 => {
                        self.file_hcontent = Some(decoder.read_fixed64()?);
                    }
                    3 => {
                        self.rtime_updated = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.game_branch_min = Some(decoder.read_string()?.to_owned());
                    }
                    5 => {
                        self.game_branch_max = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.revision {
                encoder.write_varint_field(1, i64::from(value.value()) as u64);
            }
            if let Some(value) = &self.file_hcontent {
                encoder.write_fixed64_field(2, *value);
            }
            if let Some(value) = &self.rtime_updated {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.game_branch_min {
                encoder.write_string_field(4, value);
            }
            if let Some(value) = &self.game_branch_max {
                encoder.write_string_field(5, value);
            }
        }
    }
}

/// `CPublishedFile_FileSubscribed_Notification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_FileSubscribed_Notification {
    /// Field 1.
    pub published_file_id: Option<u64>,
    /// Field 2.
    pub app_id: Option<u32>,
    /// Field 3.
    pub file_hcontent: Option<u64>,
    /// Field 4.
    pub file_size: Option<u32>,
    /// Field 5.
    pub rtime_subscribed: Option<u32>,
    /// Field 6.
    pub is_depot_content: Option<bool>,
    /// Field 7.
    pub rtime_updated: Option<u32>,
    /// Field 9.
    pub revision: Option<crate::steammessages_publishedfile_steamclient::EPublishedFileRevision>,
    /// Field 8.
    pub revisions: Vec<crate::steammessages_publishedfile_steamclient::c_published_file_file_subscribed_notification::RevisionData>,
}

impl CPublishedFile_FileSubscribed_Notification {
    /// Field 9 , or its schema default when absent.
    #[must_use]
    pub fn revision_or_default(
        &self,
    ) -> crate::steammessages_publishedfile_steamclient::EPublishedFileRevision {
        self.revision.unwrap_or(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::k_EPublishedFileRevision_Default)
    }
}

impl Message for CPublishedFile_FileSubscribed_Notification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.published_file_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.file_hcontent = Some(decoder.read_fixed64()?);
                }
                4 => {
                    self.file_size = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.rtime_subscribed = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.is_depot_content = Some(decoder.read_bool()?);
                }
                7 => {
                    self.rtime_updated = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.revision = Some(crate::steammessages_publishedfile_steamclient::EPublishedFileRevision::from(decoder.read_varint()? as i32));
                }
                8 => {
                    self.revisions.push({ let mut nested = crate::steammessages_publishedfile_steamclient::c_published_file_file_subscribed_notification::RevisionData::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.published_file_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.file_hcontent {
            encoder.write_fixed64_field(3, *value);
        }
        if let Some(value) = &self.file_size {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.rtime_subscribed {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.is_depot_content {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.rtime_updated {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.revision {
            encoder.write_varint_field(9, i64::from(value.value()) as u64);
        }
        for value in &self.revisions {
            encoder.write_message_field(8, value);
        }
    }
}

/// `CPublishedFile_FileUnsubscribed_Notification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_FileUnsubscribed_Notification {
    /// Field 1.
    pub published_file_id: Option<u64>,
    /// Field 2.
    pub app_id: Option<u32>,
}

impl Message for CPublishedFile_FileUnsubscribed_Notification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.published_file_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.published_file_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CPublishedFile_FileDeleted_Client_Notification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPublishedFile_FileDeleted_Client_Notification {
    /// Field 1.
    pub published_file_id: Option<u64>,
    /// Field 2.
    pub app_id: Option<u32>,
}

impl Message for CPublishedFile_FileDeleted_Client_Notification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.published_file_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.published_file_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_Vote_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_Vote_Response;
    const TARGET: &'static str = "PublishedFile.Vote";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_Subscribe_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_Subscribe_Response;
    const TARGET: &'static str = "PublishedFile.Subscribe";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_Unsubscribe_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_Unsubscribe_Response;
    const TARGET: &'static str = "PublishedFile.Unsubscribe";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_CanSubscribe_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_CanSubscribe_Response;
    const TARGET: &'static str = "PublishedFile.CanSubscribe";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetSubSectionData_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetSubSectionData_Response;
    const TARGET: &'static str = "PublishedFile.GetSubSectionData";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_Publish_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_Publish_Response;
    const TARGET: &'static str = "PublishedFile.Publish";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetDetails_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetDetails_Response;
    const TARGET: &'static str = "PublishedFile.GetDetails";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetItemInfo_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetItemInfo_Response;
    const TARGET: &'static str = "PublishedFile.GetItemInfo";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetUserFiles_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetUserFiles_Response;
    const TARGET: &'static str = "PublishedFile.GetUserFiles";
}

/// Unified-message target for `PublishedFile.GetUserFileCount`.
///
/// It shares its request type with `PublishedFile.GetUserFiles`, so it cannot carry an
/// `Rpc` binding of its own — call it by naming this target.
pub const TARGET_PUBLISHED_FILE_GET_USER_FILE_COUNT: &str = "PublishedFile.GetUserFileCount";

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_AreFilesInSubscriptionList_Request {
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_AreFilesInSubscriptionList_Response;
    const TARGET: &'static str = "PublishedFile.AreFilesInSubscriptionList";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_Update_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_Update_Response;
    const TARGET: &'static str = "PublishedFile.Update";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_Delete_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_Delete_Response;
    const TARGET: &'static str = "PublishedFile.Delete";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetChangeHistoryEntry_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_GetChangeHistoryEntry_Response;
    const TARGET: &'static str = "PublishedFile.GetChangeHistoryEntry";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetChangeHistory_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetChangeHistory_Response;
    const TARGET: &'static str = "PublishedFile.GetChangeHistory";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_RefreshVotingQueue_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_RefreshVotingQueue_Response;
    const TARGET: &'static str = "PublishedFile.RefreshVotingQueue";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_QueryFiles_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_QueryFiles_Response;
    const TARGET: &'static str = "PublishedFile.QueryFiles";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_AddAppRelationship_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_AddAppRelationship_Response;
    const TARGET: &'static str = "PublishedFile.AddAppRelationship";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_RemoveAppRelationship_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_RemoveAppRelationship_Response;
    const TARGET: &'static str = "PublishedFile.RemoveAppRelationship";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetAppRelationships_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetAppRelationships_Response;
    const TARGET: &'static str = "PublishedFile.GetAppRelationships";
}

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetAppRelationshipsBatched_Request {
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_GetAppRelationshipsBatched_Response;
    const TARGET: &'static str = "PublishedFile.GetAppRelationshipsBatched";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_StartPlaytimeTracking_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_StartPlaytimeTracking_Response;
    const TARGET: &'static str = "PublishedFile.StartPlaytimeTracking";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_StopPlaytimeTracking_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_StopPlaytimeTracking_Response;
    const TARGET: &'static str = "PublishedFile.StopPlaytimeTracking";
}

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_StopPlaytimeTrackingForAllAppItems_Request {
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_StopPlaytimeTrackingForAllAppItems_Response;
    const TARGET: &'static str = "PublishedFile.StopPlaytimeTrackingForAllAppItems";
}

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_SetPlaytimeForControllerConfigs_Request {
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_SetPlaytimeForControllerConfigs_Response;
    const TARGET: &'static str = "PublishedFile.SetPlaytimeForControllerConfigs";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_AddChild_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_AddChild_Response;
    const TARGET: &'static str = "PublishedFile.AddChild";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_RemoveChild_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_RemoveChild_Response;
    const TARGET: &'static str = "PublishedFile.RemoveChild";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_SetCollectionChildren_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_SetCollectionChildren_Response;
    const TARGET: &'static str = "PublishedFile.SetCollectionChildren";
}

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_SetSubscriptionListFromCollection_Request {
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_SetSubscriptionListFromCollection_Response;
    const TARGET: &'static str = "PublishedFile.SetSubscriptionListFromCollection";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetUserVoteSummary_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetUserVoteSummary_Response;
    const TARGET: &'static str = "PublishedFile.GetUserVoteSummary";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetItemChanges_Request
{
    type Response =
        crate::steammessages_publishedfile_steamclient::CPublishedFile_GetItemChanges_Response;
    const TARGET: &'static str = "PublishedFile.GetItemChanges";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_GetContentDescriptors_Request
{
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_GetContentDescriptors_Response;
    const TARGET: &'static str = "PublishedFile.GetContentDescriptors";
}

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_UpdateContentDescriptors_Request {
    type Response = crate::steammessages_publishedfile_steamclient::CPublishedFile_UpdateContentDescriptors_Response;
    const TARGET: &'static str = "PublishedFile.UpdateContentDescriptors";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_FileSubscribed_Notification
{
    type Response = crate::steammessages_unified_base_steamclient::NoResponse;
    const TARGET: &'static str = "PublishedFileClient.NotifyFileSubscribed";
}

impl tapline_wire::Rpc
    for crate::steammessages_publishedfile_steamclient::CPublishedFile_FileUnsubscribed_Notification
{
    type Response = crate::steammessages_unified_base_steamclient::NoResponse;
    const TARGET: &'static str = "PublishedFileClient.NotifyFileUnsubscribed";
}

impl tapline_wire::Rpc for crate::steammessages_publishedfile_steamclient::CPublishedFile_FileDeleted_Client_Notification {
    type Response = crate::steammessages_unified_base_steamclient::NoResponse;
    const TARGET: &'static str = "PublishedFileClient.NotifyFileDeleted";
}
