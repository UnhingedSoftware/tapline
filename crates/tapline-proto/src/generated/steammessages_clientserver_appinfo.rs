//! Generated from `steammessages_clientserver_appinfo.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `CMsgClientAppInfoUpdate` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAppInfoUpdate {
    /// Field 1.
    pub last_changenumber: Option<u32>,
    /// Field 2.
    pub send_changelist: Option<bool>,
}

impl Message for CMsgClientAppInfoUpdate {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.last_changenumber = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.send_changelist = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.last_changenumber {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.send_changelist {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// `CMsgClientAppInfoChanges` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAppInfoChanges {
    /// Field 1.
    pub current_change_number: Option<u32>,
    /// Field 2.
    pub force_full_update: Option<bool>,
    /// Field 3.
    pub app_i_ds: Vec<u32>,
}

impl Message for CMsgClientAppInfoChanges {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.current_change_number = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.force_full_update = Some(decoder.read_bool()?);
                }
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.app_i_ds,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.current_change_number {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.force_full_update {
            encoder.write_bool_field(2, *value);
        }
        for value in &self.app_i_ds {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientAppInfoRequest`].
pub mod c_msg_client_app_info_request {
    use super::*;

    /// `App` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct App {
        /// Field 1.
        pub app_id: Option<u32>,
        /// Field 2.
        pub section_flags: Option<u32>,
        /// Field 3.
        pub section_crc: Vec<u32>,
    }

    impl Message for App {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.app_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.section_flags = Some(decoder.read_varint()? as u32);
                    }
                    3 => decoder.read_maybe_packed(
                        key.wire_type,
                        &mut self.section_crc,
                        |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                    )?,
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.app_id {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.section_flags {
                encoder.write_varint_field(2, u64::from(*value));
            }
            for value in &self.section_crc {
                encoder.write_varint_field(3, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientAppInfoRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAppInfoRequest {
    /// Field 1.
    pub apps: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_app_info_request::App>,
    /// Field 2.
    pub supports_batches: Option<bool>,
}

impl CMsgClientAppInfoRequest {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn supports_batches_or_default(&self) -> bool {
        self.supports_batches.unwrap_or(false)
    }
}

impl Message for CMsgClientAppInfoRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.apps.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_app_info_request::App::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.supports_batches = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.apps {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.supports_batches {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// `CMsgClientPICSChangesSinceRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSChangesSinceRequest {
    /// Field 1.
    pub since_change_number: Option<u32>,
    /// Field 2.
    pub send_app_info_changes: Option<bool>,
    /// Field 3.
    pub send_package_info_changes: Option<bool>,
    /// Field 4.
    pub num_app_info_cached: Option<u32>,
    /// Field 5.
    pub num_package_info_cached: Option<u32>,
}

impl Message for CMsgClientPICSChangesSinceRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.since_change_number = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.send_app_info_changes = Some(decoder.read_bool()?);
                }
                3 => {
                    self.send_package_info_changes = Some(decoder.read_bool()?);
                }
                4 => {
                    self.num_app_info_cached = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.num_package_info_cached = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.since_change_number {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.send_app_info_changes {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.send_package_info_changes {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.num_app_info_cached {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.num_package_info_cached {
            encoder.write_varint_field(5, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientPICSChangesSinceResponse`].
pub mod c_msg_client_pics_changes_since_response {
    use super::*;

    /// `PackageChange` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PackageChange {
        /// Field 1.
        pub packageid: Option<u32>,
        /// Field 2.
        pub change_number: Option<u32>,
        /// Field 3.
        pub needs_token: Option<bool>,
    }

    impl Message for PackageChange {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.packageid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.change_number = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.needs_token = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.packageid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.change_number {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.needs_token {
                encoder.write_bool_field(3, *value);
            }
        }
    }

    /// `AppChange` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AppChange {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub change_number: Option<u32>,
        /// Field 3.
        pub needs_token: Option<bool>,
    }

    impl Message for AppChange {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.change_number = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.needs_token = Some(decoder.read_bool()?);
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
            if let Some(value) = &self.change_number {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.needs_token {
                encoder.write_bool_field(3, *value);
            }
        }
    }
}

/// `CMsgClientPICSChangesSinceResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSChangesSinceResponse {
    /// Field 1.
    pub current_change_number: Option<u32>,
    /// Field 2.
    pub since_change_number: Option<u32>,
    /// Field 3.
    pub force_full_update: Option<bool>,
    /// Field 4.
    pub package_changes: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_changes_since_response::PackageChange>,
    /// Field 5.
    pub app_changes: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_changes_since_response::AppChange>,
    /// Field 6.
    pub force_full_app_update: Option<bool>,
    /// Field 7.
    pub force_full_package_update: Option<bool>,
}

impl Message for CMsgClientPICSChangesSinceResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.current_change_number = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.since_change_number = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.force_full_update = Some(decoder.read_bool()?);
                }
                4 => {
                    self.package_changes.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_changes_since_response::PackageChange::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                5 => {
                    self.app_changes.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_changes_since_response::AppChange::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                6 => {
                    self.force_full_app_update = Some(decoder.read_bool()?);
                }
                7 => {
                    self.force_full_package_update = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.current_change_number {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.since_change_number {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.force_full_update {
            encoder.write_bool_field(3, *value);
        }
        for value in &self.package_changes {
            encoder.write_message_field(4, value);
        }
        for value in &self.app_changes {
            encoder.write_message_field(5, value);
        }
        if let Some(value) = &self.force_full_app_update {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.force_full_package_update {
            encoder.write_bool_field(7, *value);
        }
    }
}

/// Types nested inside [`CMsgClientPICSProductInfoRequest`].
pub mod c_msg_client_pics_product_info_request {
    use super::*;

    /// `AppInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AppInfo {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub access_token: Option<u64>,
        /// Field 3.
        pub only_public_obsolete: Option<bool>,
    }

    impl Message for AppInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.access_token = Some(decoder.read_varint()?);
                    }
                    3 => {
                        self.only_public_obsolete = Some(decoder.read_bool()?);
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
            if let Some(value) = &self.access_token {
                encoder.write_varint_field(2, *value);
            }
            if let Some(value) = &self.only_public_obsolete {
                encoder.write_bool_field(3, *value);
            }
        }
    }

    /// `PackageInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PackageInfo {
        /// Field 1.
        pub packageid: Option<u32>,
        /// Field 2.
        pub access_token: Option<u64>,
    }

    impl Message for PackageInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.packageid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.access_token = Some(decoder.read_varint()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.packageid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.access_token {
                encoder.write_varint_field(2, *value);
            }
        }
    }
}

/// `CMsgClientPICSProductInfoRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSProductInfoRequest {
    /// Field 1.
    pub packages: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_request::PackageInfo>,
    /// Field 2.
    pub apps: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_request::AppInfo>,
    /// Field 3.
    pub meta_data_only: Option<bool>,
    /// Field 4.
    pub num_prev_failed: Option<u32>,
    /// Field 5.
    pub obsolete_supports_package_tokens: Option<u32>,
    /// Field 6.
    pub sequence_number: Option<u32>,
    /// Field 7.
    pub single_response: Option<bool>,
}

impl Message for CMsgClientPICSProductInfoRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.packages.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_request::PackageInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.apps.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_request::AppInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                3 => {
                    self.meta_data_only = Some(decoder.read_bool()?);
                }
                4 => {
                    self.num_prev_failed = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.obsolete_supports_package_tokens = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.sequence_number = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.single_response = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.packages {
            encoder.write_message_field(1, value);
        }
        for value in &self.apps {
            encoder.write_message_field(2, value);
        }
        if let Some(value) = &self.meta_data_only {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.num_prev_failed {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.obsolete_supports_package_tokens {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.sequence_number {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.single_response {
            encoder.write_bool_field(7, *value);
        }
    }
}

/// Types nested inside [`CMsgClientPICSProductInfoResponse`].
pub mod c_msg_client_pics_product_info_response {
    use super::*;

    /// `AppInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AppInfo {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub change_number: Option<u32>,
        /// Field 3.
        pub missing_token: Option<bool>,
        /// Field 4.
        pub sha: Option<Vec<u8>>,
        /// Field 5.
        pub buffer: Option<Vec<u8>>,
        /// Field 6.
        pub only_public: Option<bool>,
        /// Field 7.
        pub size: Option<u32>,
    }

    impl Message for AppInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.change_number = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.missing_token = Some(decoder.read_bool()?);
                    }
                    4 => {
                        self.sha = Some(decoder.read_bytes()?.to_vec());
                    }
                    5 => {
                        self.buffer = Some(decoder.read_bytes()?.to_vec());
                    }
                    6 => {
                        self.only_public = Some(decoder.read_bool()?);
                    }
                    7 => {
                        self.size = Some(decoder.read_varint()? as u32);
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
            if let Some(value) = &self.change_number {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.missing_token {
                encoder.write_bool_field(3, *value);
            }
            if let Some(value) = &self.sha {
                encoder.write_bytes_field(4, value);
            }
            if let Some(value) = &self.buffer {
                encoder.write_bytes_field(5, value);
            }
            if let Some(value) = &self.only_public {
                encoder.write_bool_field(6, *value);
            }
            if let Some(value) = &self.size {
                encoder.write_varint_field(7, u64::from(*value));
            }
        }
    }

    /// `PackageInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PackageInfo {
        /// Field 1.
        pub packageid: Option<u32>,
        /// Field 2.
        pub change_number: Option<u32>,
        /// Field 3.
        pub missing_token: Option<bool>,
        /// Field 4.
        pub sha: Option<Vec<u8>>,
        /// Field 5.
        pub buffer: Option<Vec<u8>>,
        /// Field 6.
        pub size: Option<u32>,
    }

    impl Message for PackageInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.packageid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.change_number = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.missing_token = Some(decoder.read_bool()?);
                    }
                    4 => {
                        self.sha = Some(decoder.read_bytes()?.to_vec());
                    }
                    5 => {
                        self.buffer = Some(decoder.read_bytes()?.to_vec());
                    }
                    6 => {
                        self.size = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.packageid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.change_number {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.missing_token {
                encoder.write_bool_field(3, *value);
            }
            if let Some(value) = &self.sha {
                encoder.write_bytes_field(4, value);
            }
            if let Some(value) = &self.buffer {
                encoder.write_bytes_field(5, value);
            }
            if let Some(value) = &self.size {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientPICSProductInfoResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSProductInfoResponse {
    /// Field 1.
    pub apps: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_response::AppInfo>,
    /// Field 2.
    pub unknown_appids: Vec<u32>,
    /// Field 3.
    pub packages: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_response::PackageInfo>,
    /// Field 4.
    pub unknown_packageids: Vec<u32>,
    /// Field 5.
    pub meta_data_only: Option<bool>,
    /// Field 6.
    pub response_pending: Option<bool>,
    /// Field 7.
    pub http_min_size: Option<u32>,
    /// Field 8.
    pub http_host: Option<String>,
}

impl Message for CMsgClientPICSProductInfoResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.apps.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_response::AppInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.unknown_appids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                3 => {
                    self.packages.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_product_info_response::PackageInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                4 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.unknown_packageids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                5 => {
                    self.meta_data_only = Some(decoder.read_bool()?);
                }
                6 => {
                    self.response_pending = Some(decoder.read_bool()?);
                }
                7 => {
                    self.http_min_size = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.http_host = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.apps {
            encoder.write_message_field(1, value);
        }
        for value in &self.unknown_appids {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.packages {
            encoder.write_message_field(3, value);
        }
        for value in &self.unknown_packageids {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.meta_data_only {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.response_pending {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.http_min_size {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.http_host {
            encoder.write_string_field(8, value);
        }
    }
}

/// `CMsgClientPICSAccessTokenRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSAccessTokenRequest {
    /// Field 1.
    pub packageids: Vec<u32>,
    /// Field 2.
    pub appids: Vec<u32>,
}

impl Message for CMsgClientPICSAccessTokenRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.packageids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.appids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.packageids {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.appids {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientPICSAccessTokenResponse`].
pub mod c_msg_client_pics_access_token_response {
    use super::*;

    /// `PackageToken` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PackageToken {
        /// Field 1.
        pub packageid: Option<u32>,
        /// Field 2.
        pub access_token: Option<u64>,
    }

    impl Message for PackageToken {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.packageid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.access_token = Some(decoder.read_varint()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.packageid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.access_token {
                encoder.write_varint_field(2, *value);
            }
        }
    }

    /// `AppToken` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AppToken {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub access_token: Option<u64>,
    }

    impl Message for AppToken {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.access_token = Some(decoder.read_varint()?);
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
            if let Some(value) = &self.access_token {
                encoder.write_varint_field(2, *value);
            }
        }
    }
}

/// `CMsgClientPICSAccessTokenResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSAccessTokenResponse {
    /// Field 1.
    pub package_access_tokens: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_access_token_response::PackageToken>,
    /// Field 2.
    pub package_denied_tokens: Vec<u32>,
    /// Field 3.
    pub app_access_tokens: Vec<crate::steammessages_clientserver_appinfo::c_msg_client_pics_access_token_response::AppToken>,
    /// Field 4.
    pub app_denied_tokens: Vec<u32>,
}

impl Message for CMsgClientPICSAccessTokenResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.package_access_tokens.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_access_token_response::PackageToken::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.package_denied_tokens,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                3 => {
                    self.app_access_tokens.push({ let mut nested = crate::steammessages_clientserver_appinfo::c_msg_client_pics_access_token_response::AppToken::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                4 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.app_denied_tokens,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.package_access_tokens {
            encoder.write_message_field(1, value);
        }
        for value in &self.package_denied_tokens {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.app_access_tokens {
            encoder.write_message_field(3, value);
        }
        for value in &self.app_denied_tokens {
            encoder.write_varint_field(4, u64::from(*value));
        }
    }
}

/// `CMsgClientPICSPrivateBetaRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSPrivateBetaRequest {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub access_token: Option<u64>,
    /// Field 3.
    pub beta_name: Option<String>,
    /// Field 4.
    pub password_hash: Option<Vec<u8>>,
}

impl Message for CMsgClientPICSPrivateBetaRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.access_token = Some(decoder.read_varint()?);
                }
                3 => {
                    self.beta_name = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.password_hash = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.access_token {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.beta_name {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.password_hash {
            encoder.write_bytes_field(4, value);
        }
    }
}

/// `CMsgClientPICSPrivateBetaResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPICSPrivateBetaResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub depot_section: Option<Vec<u8>>,
}

impl CMsgClientPICSPrivateBetaResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientPICSPrivateBetaResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.depot_section = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.eresult {
            encoder.write_int32_field(1, *value);
        }
        if let Some(value) = &self.depot_section {
            encoder.write_bytes_field(2, value);
        }
    }
}
