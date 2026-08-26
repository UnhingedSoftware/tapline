//! Generated from `steammessages_contentsystem.steamclient.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EAppContentDetectionType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAppContentDetectionType(pub i32);

impl EAppContentDetectionType {
    /// `k_EAppContentDetectionType_None` = `0`
    pub const k_EAppContentDetectionType_None: Self = Self(0);
    /// `k_EAppContentDetectionType_AntiCheat` = `1`
    pub const k_EAppContentDetectionType_AntiCheat: Self = Self(1);
    /// `k_EAppContentDetectionType_GameEngine` = `2`
    pub const k_EAppContentDetectionType_GameEngine: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAppContentDetectionType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `CContentServerDirectory_ConnectedSteamPipeServerInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_ConnectedSteamPipeServerInfo {
    /// Field 1.
    pub r#type: Option<String>,
    /// Field 2.
    pub source_id: Option<i32>,
    /// Field 3.
    pub hostname: Option<String>,
}

impl Message for CContentServerDirectory_ConnectedSteamPipeServerInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.r#type = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.source_id = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.hostname = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.r#type {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.source_id {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.hostname {
            encoder.write_string_field(3, value);
        }
    }
}

/// `CContentServerDirectory_GetServersForSteamPipe_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetServersForSteamPipe_Request {
    /// Field 1.
    pub cell_id: Option<u32>,
    /// Field 2.
    pub max_servers: Option<u32>,
    /// Field 3.
    pub ip_override: Option<String>,
    /// Field 4.
    pub launcher_type: Option<i32>,
    /// Field 5.
    pub ipv6_public: Option<String>,
    /// Field 6.
    pub current_connections: Vec<crate::steammessages_contentsystem_steamclient::CContentServerDirectory_ConnectedSteamPipeServerInfo>,
}

impl CContentServerDirectory_GetServersForSteamPipe_Request {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn max_servers_or_default(&self) -> u32 {
        self.max_servers.unwrap_or(20_u32)
    }
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn launcher_type_or_default(&self) -> i32 {
        self.launcher_type.unwrap_or(0_i32)
    }
}

impl Message for CContentServerDirectory_GetServersForSteamPipe_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.cell_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.max_servers = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.ip_override = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.launcher_type = Some(decoder.read_varint()? as i32);
                }
                5 => {
                    self.ipv6_public = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.current_connections.push({ let mut nested = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_ConnectedSteamPipeServerInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.cell_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.max_servers {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.ip_override {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.launcher_type {
            encoder.write_int32_field(4, *value);
        }
        if let Some(value) = &self.ipv6_public {
            encoder.write_string_field(5, value);
        }
        for value in &self.current_connections {
            encoder.write_message_field(6, value);
        }
    }
}

/// `CContentServerDirectory_ServerInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_ServerInfo {
    /// Field 1.
    pub r#type: Option<String>,
    /// Field 2.
    pub source_id: Option<i32>,
    /// Field 3.
    pub cell_id: Option<i32>,
    /// Field 4.
    pub load: Option<i32>,
    /// Field 5.
    pub weighted_load: Option<f32>,
    /// Field 6.
    pub num_entries_in_client_list: Option<i32>,
    /// Field 7.
    pub steam_china_only: Option<bool>,
    /// Field 8.
    pub host: Option<String>,
    /// Field 9.
    pub vhost: Option<String>,
    /// Field 10.
    pub use_as_proxy: Option<bool>,
    /// Field 11.
    pub proxy_request_path_template: Option<String>,
    /// Field 12.
    pub https_support: Option<String>,
    /// Field 13.
    pub allowed_app_ids: Vec<u32>,
    /// Field 15.
    pub priority_class: Option<u32>,
    /// Field 16.
    pub bypass_proxies_of_type: Vec<String>,
    /// Field 17.
    pub group: Option<String>,
}

impl Message for CContentServerDirectory_ServerInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.r#type = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.source_id = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.cell_id = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.load = Some(decoder.read_varint()? as i32);
                }
                5 => {
                    self.weighted_load = Some(decoder.read_float()?);
                }
                6 => {
                    self.num_entries_in_client_list = Some(decoder.read_varint()? as i32);
                }
                7 => {
                    self.steam_china_only = Some(decoder.read_bool()?);
                }
                8 => {
                    self.host = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.vhost = Some(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.use_as_proxy = Some(decoder.read_bool()?);
                }
                11 => {
                    self.proxy_request_path_template = Some(decoder.read_string()?.to_owned());
                }
                12 => {
                    self.https_support = Some(decoder.read_string()?.to_owned());
                }
                13 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.allowed_app_ids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                15 => {
                    self.priority_class = Some(decoder.read_varint()? as u32);
                }
                16 => {
                    self.bypass_proxies_of_type
                        .push(decoder.read_string()?.to_owned());
                }
                17 => {
                    self.group = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.r#type {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.source_id {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.cell_id {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.load {
            encoder.write_int32_field(4, *value);
        }
        if let Some(value) = &self.weighted_load {
            encoder.write_float_field(5, *value);
        }
        if let Some(value) = &self.num_entries_in_client_list {
            encoder.write_int32_field(6, *value);
        }
        if let Some(value) = &self.steam_china_only {
            encoder.write_bool_field(7, *value);
        }
        if let Some(value) = &self.host {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.vhost {
            encoder.write_string_field(9, value);
        }
        if let Some(value) = &self.use_as_proxy {
            encoder.write_bool_field(10, *value);
        }
        if let Some(value) = &self.proxy_request_path_template {
            encoder.write_string_field(11, value);
        }
        if let Some(value) = &self.https_support {
            encoder.write_string_field(12, value);
        }
        for value in &self.allowed_app_ids {
            encoder.write_varint_field(13, u64::from(*value));
        }
        if let Some(value) = &self.priority_class {
            encoder.write_varint_field(15, u64::from(*value));
        }
        for value in &self.bypass_proxies_of_type {
            encoder.write_string_field(16, value);
        }
        if let Some(value) = &self.group {
            encoder.write_string_field(17, value);
        }
    }
}

/// `CContentServerDirectory_GetServersForSteamPipe_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetServersForSteamPipe_Response {
    /// Field 1.
    pub servers:
        Vec<crate::steammessages_contentsystem_steamclient::CContentServerDirectory_ServerInfo>,
    /// Field 2.
    pub no_change: Option<bool>,
}

impl Message for CContentServerDirectory_GetServersForSteamPipe_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.servers.push({ let mut nested = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_ServerInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.no_change = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.servers {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.no_change {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// `CContentServerDirectory_GetDepotPatchInfo_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetDepotPatchInfo_Request {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub depotid: Option<u32>,
    /// Field 3.
    pub source_manifestid: Option<u64>,
    /// Field 4.
    pub target_manifestid: Option<u64>,
}

impl Message for CContentServerDirectory_GetDepotPatchInfo_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.depotid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.source_manifestid = Some(decoder.read_varint()?);
                }
                4 => {
                    self.target_manifestid = Some(decoder.read_varint()?);
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
        if let Some(value) = &self.depotid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.source_manifestid {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.target_manifestid {
            encoder.write_varint_field(4, *value);
        }
    }
}

/// `CContentServerDirectory_GetDepotPatchInfo_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetDepotPatchInfo_Response {
    /// Field 1.
    pub is_available: Option<bool>,
    /// Field 2.
    pub patch_size: Option<u64>,
    /// Field 3.
    pub patched_chunks_size: Option<u64>,
}

impl Message for CContentServerDirectory_GetDepotPatchInfo_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.is_available = Some(decoder.read_bool()?);
                }
                2 => {
                    self.patch_size = Some(decoder.read_varint()?);
                }
                3 => {
                    self.patched_chunks_size = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.is_available {
            encoder.write_bool_field(1, *value);
        }
        if let Some(value) = &self.patch_size {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.patched_chunks_size {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CContentServerDirectory_GetClientUpdateHosts_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetClientUpdateHosts_Request {
    /// Field 1.
    pub cached_signature: Option<String>,
}

impl Message for CContentServerDirectory_GetClientUpdateHosts_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.cached_signature = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.cached_signature {
            encoder.write_string_field(1, value);
        }
    }
}

/// `CContentServerDirectory_GetClientUpdateHosts_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetClientUpdateHosts_Response {
    /// Field 1.
    pub hosts_kv: Option<String>,
    /// Field 2.
    pub valid_until_time: Option<u64>,
    /// Field 3.
    pub ip_country: Option<String>,
}

impl Message for CContentServerDirectory_GetClientUpdateHosts_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.hosts_kv = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.valid_until_time = Some(decoder.read_varint()?);
                }
                3 => {
                    self.ip_country = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.hosts_kv {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.valid_until_time {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.ip_country {
            encoder.write_string_field(3, value);
        }
    }
}

/// `CContentServerDirectory_GetManifestRequestCode_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetManifestRequestCode_Request {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub depot_id: Option<u32>,
    /// Field 3.
    pub manifest_id: Option<u64>,
    /// Field 4.
    pub app_branch: Option<String>,
    /// Field 5.
    pub branch_password_hash: Option<String>,
}

impl Message for CContentServerDirectory_GetManifestRequestCode_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.manifest_id = Some(decoder.read_varint()?);
                }
                4 => {
                    self.app_branch = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.branch_password_hash = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.manifest_id {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.app_branch {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.branch_password_hash {
            encoder.write_string_field(5, value);
        }
    }
}

/// `CContentServerDirectory_GetManifestRequestCode_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetManifestRequestCode_Response {
    /// Field 1.
    pub manifest_request_code: Option<u64>,
}

impl Message for CContentServerDirectory_GetManifestRequestCode_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.manifest_request_code = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.manifest_request_code {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CContentServerDirectory_GetCDNAuthToken_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetCDNAuthToken_Request {
    /// Field 1.
    pub depot_id: Option<u32>,
    /// Field 2.
    pub host_name: Option<String>,
    /// Field 3.
    pub app_id: Option<u32>,
}

impl Message for CContentServerDirectory_GetCDNAuthToken_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.host_name = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.host_name {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CContentServerDirectory_GetCDNAuthToken_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetCDNAuthToken_Response {
    /// Field 1.
    pub token: Option<String>,
    /// Field 2.
    pub expiration_time: Option<u32>,
}

impl Message for CContentServerDirectory_GetCDNAuthToken_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.expiration_time = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.expiration_time {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CContentServerDirectory_RequestPeerContentServer_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_RequestPeerContentServer_Request {
    /// Field 1.
    pub remote_client_id: Option<u64>,
    /// Field 2.
    pub steamid: Option<u64>,
    /// Field 3.
    pub server_remote_client_id: Option<u64>,
    /// Field 4.
    pub app_id: Option<u32>,
    /// Field 5.
    pub current_build_id: Option<u32>,
}

impl Message for CContentServerDirectory_RequestPeerContentServer_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.remote_client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.steamid = Some(decoder.read_varint()?);
                }
                3 => {
                    self.server_remote_client_id = Some(decoder.read_varint()?);
                }
                4 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.current_build_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.remote_client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.server_remote_client_id {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.current_build_id {
            encoder.write_varint_field(5, u64::from(*value));
        }
    }
}

/// `CContentServerDirectory_RequestPeerContentServer_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_RequestPeerContentServer_Response {
    /// Field 1.
    pub server_port: Option<u32>,
    /// Field 2.
    pub installed_depots: Vec<u32>,
    /// Field 3.
    pub access_token: Option<u64>,
}

impl Message for CContentServerDirectory_RequestPeerContentServer_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.server_port = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.installed_depots,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                3 => {
                    self.access_token = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.server_port {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.installed_depots {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.access_token {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CContentServerDirectory_GetPeerContentInfo_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetPeerContentInfo_Request {
    /// Field 1.
    pub remote_client_id: Option<u64>,
    /// Field 2.
    pub steamid: Option<u64>,
    /// Field 3.
    pub server_remote_client_id: Option<u64>,
}

impl Message for CContentServerDirectory_GetPeerContentInfo_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.remote_client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.steamid = Some(decoder.read_varint()?);
                }
                3 => {
                    self.server_remote_client_id = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.remote_client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.server_remote_client_id {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CContentServerDirectory_GetPeerContentInfo_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CContentServerDirectory_GetPeerContentInfo_Response {
    /// Field 1.
    pub appids: Vec<u32>,
    /// Field 2.
    pub ip_public: Option<String>,
}

impl Message for CContentServerDirectory_GetPeerContentInfo_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.appids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                2 => {
                    self.ip_public = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.appids {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.ip_public {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CDepotContentDetection_GetAllDetectedAppContent_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CDepotContentDetection_GetAllDetectedAppContent_Request {
    /// Field 1.
    pub detection_type:
        Option<crate::steammessages_contentsystem_steamclient::EAppContentDetectionType>,
}

impl CDepotContentDetection_GetAllDetectedAppContent_Request {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn detection_type_or_default(
        &self,
    ) -> crate::steammessages_contentsystem_steamclient::EAppContentDetectionType {
        self.detection_type.unwrap_or(crate::steammessages_contentsystem_steamclient::EAppContentDetectionType::k_EAppContentDetectionType_None)
    }
}

impl Message for CDepotContentDetection_GetAllDetectedAppContent_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.detection_type = Some(crate::steammessages_contentsystem_steamclient::EAppContentDetectionType::from(decoder.read_varint()? as i32));
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.detection_type {
            encoder.write_varint_field(1, i64::from(value.value()) as u64);
        }
    }
}

/// `DetectedAppContent` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DetectedAppContent {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub depot_id: Option<u32>,
    /// Field 3.
    pub detected_content: Option<i32>,
}

impl Message for DetectedAppContent {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.detected_content = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.detected_content {
            encoder.write_int32_field(3, *value);
        }
    }
}

/// `CDepotContentDetection_GetAllDetectedAppContent_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CDepotContentDetection_GetAllDetectedAppContent_Response {
    /// Field 1.
    pub detected_app_content:
        Vec<crate::steammessages_contentsystem_steamclient::DetectedAppContent>,
}

impl Message for CDepotContentDetection_GetAllDetectedAppContent_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.detected_app_content.push({ let mut nested = crate::steammessages_contentsystem_steamclient::DetectedAppContent::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.detected_app_content {
            encoder.write_message_field(1, value);
        }
    }
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetServersForSteamPipe_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetServersForSteamPipe_Response;
    const TARGET: &'static str = "ContentServerDirectory.GetServersForSteamPipe";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetDepotPatchInfo_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetDepotPatchInfo_Response;
    const TARGET: &'static str = "ContentServerDirectory.GetDepotPatchInfo";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetClientUpdateHosts_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetClientUpdateHosts_Response;
    const TARGET: &'static str = "ContentServerDirectory.GetClientUpdateHosts";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetManifestRequestCode_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetManifestRequestCode_Response;
    const TARGET: &'static str = "ContentServerDirectory.GetManifestRequestCode";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetCDNAuthToken_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetCDNAuthToken_Response;
    const TARGET: &'static str = "ContentServerDirectory.GetCDNAuthToken";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_RequestPeerContentServer_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_RequestPeerContentServer_Response;
    const TARGET: &'static str = "ContentServerDirectory.RequestPeerContentServer";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetPeerContentInfo_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CContentServerDirectory_GetPeerContentInfo_Response;
    const TARGET: &'static str = "ContentServerDirectory.GetPeerContentInfo";
}

impl tapline_wire::Rpc for crate::steammessages_contentsystem_steamclient::CDepotContentDetection_GetAllDetectedAppContent_Request {
    type Response = crate::steammessages_contentsystem_steamclient::CDepotContentDetection_GetAllDetectedAppContent_Response;
    const TARGET: &'static str = "DepotContentDetection.GetAllDetectedAppContent";
}
