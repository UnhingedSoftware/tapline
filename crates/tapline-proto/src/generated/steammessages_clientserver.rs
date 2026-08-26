//! Generated from `steammessages_clientserver.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `CMsgClientRegisterAuthTicketWithCM` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRegisterAuthTicketWithCM {
    /// Field 1.
    pub protocol_version: Option<u32>,
    /// Field 3.
    pub ticket: Option<Vec<u8>>,
    /// Field 4.
    pub client_instance_id: Option<u64>,
}

impl Message for CMsgClientRegisterAuthTicketWithCM {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.protocol_version = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.ticket = Some(decoder.read_bytes()?.to_vec());
                }
                4 => {
                    self.client_instance_id = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.protocol_version {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.ticket {
            encoder.write_bytes_field(3, value);
        }
        if let Some(value) = &self.client_instance_id {
            encoder.write_varint_field(4, *value);
        }
    }
}

/// `CMsgClientTicketAuthComplete` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientTicketAuthComplete {
    /// Field 1.
    pub steam_id: Option<u64>,
    /// Field 2.
    pub game_id: Option<u64>,
    /// Field 3.
    pub estate: Option<u32>,
    /// Field 4.
    pub eauth_session_response: Option<u32>,
    /// Field 5.
    pub deprecated_ticket: Option<Vec<u8>>,
    /// Field 6.
    pub ticket_crc: Option<u32>,
    /// Field 7.
    pub ticket_sequence: Option<u32>,
    /// Field 8.
    pub owner_steam_id: Option<u64>,
}

impl Message for CMsgClientTicketAuthComplete {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.game_id = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.estate = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.eauth_session_response = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.deprecated_ticket = Some(decoder.read_bytes()?.to_vec());
                }
                6 => {
                    self.ticket_crc = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.ticket_sequence = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.owner_steam_id = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.game_id {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.estate {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.eauth_session_response {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.deprecated_ticket {
            encoder.write_bytes_field(5, value);
        }
        if let Some(value) = &self.ticket_crc {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.ticket_sequence {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.owner_steam_id {
            encoder.write_fixed64_field(8, *value);
        }
    }
}

/// `CMsgClientP2PConnectionInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientP2PConnectionInfo {
    /// Field 1.
    pub steam_id_dest: Option<u64>,
    /// Field 2.
    pub steam_id_src: Option<u64>,
    /// Field 3.
    pub app_id: Option<u32>,
    /// Field 4.
    pub candidate: Option<Vec<u8>>,
    /// Field 5.
    pub legacy_connection_id_src: Option<u64>,
    /// Field 6.
    pub rendezvous: Option<Vec<u8>>,
    /// Field 7.
    pub app_id_secondary: Option<u32>,
}

impl Message for CMsgClientP2PConnectionInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id_dest = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.steam_id_src = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.candidate = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.legacy_connection_id_src = Some(decoder.read_fixed64()?);
                }
                6 => {
                    self.rendezvous = Some(decoder.read_bytes()?.to_vec());
                }
                7 => {
                    self.app_id_secondary = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id_dest {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.steam_id_src {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.candidate {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.legacy_connection_id_src {
            encoder.write_fixed64_field(5, *value);
        }
        if let Some(value) = &self.rendezvous {
            encoder.write_bytes_field(6, value);
        }
        if let Some(value) = &self.app_id_secondary {
            encoder.write_varint_field(7, u64::from(*value));
        }
    }
}

/// `CMsgClientP2PConnectionFailInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientP2PConnectionFailInfo {
    /// Field 1.
    pub steam_id_dest: Option<u64>,
    /// Field 2.
    pub steam_id_src: Option<u64>,
    /// Field 3.
    pub app_id: Option<u32>,
    /// Field 4.
    pub ep2p_session_error: Option<u32>,
    /// Field 5.
    pub connection_id_dest: Option<u64>,
    /// Field 7.
    pub close_reason: Option<u32>,
    /// Field 8.
    pub close_message: Option<String>,
}

impl Message for CMsgClientP2PConnectionFailInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id_dest = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.steam_id_src = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.ep2p_session_error = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.connection_id_dest = Some(decoder.read_fixed64()?);
                }
                7 => {
                    self.close_reason = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.close_message = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id_dest {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.steam_id_src {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.ep2p_session_error {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.connection_id_dest {
            encoder.write_fixed64_field(5, *value);
        }
        if let Some(value) = &self.close_reason {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.close_message {
            encoder.write_string_field(8, value);
        }
    }
}

/// `CMsgClientNetworkingCertRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientNetworkingCertRequest {
    /// Field 2.
    pub key_data: Option<Vec<u8>>,
    /// Field 3.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientNetworkingCertRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                2 => {
                    self.key_data = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.key_data {
            encoder.write_bytes_field(2, value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CMsgClientNetworkingCertReply` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientNetworkingCertReply {
    /// Field 4.
    pub cert: Option<Vec<u8>>,
    /// Field 5.
    pub ca_key_id: Option<u64>,
    /// Field 6.
    pub ca_signature: Option<Vec<u8>>,
}

impl Message for CMsgClientNetworkingCertReply {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                4 => {
                    self.cert = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.ca_key_id = Some(decoder.read_fixed64()?);
                }
                6 => {
                    self.ca_signature = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.cert {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.ca_key_id {
            encoder.write_fixed64_field(5, *value);
        }
        if let Some(value) = &self.ca_signature {
            encoder.write_bytes_field(6, value);
        }
    }
}

/// `CMsgClientNetworkingMobileCertRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientNetworkingMobileCertRequest {
    /// Field 1.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientNetworkingMobileCertRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
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
    }
}

/// `CMsgClientNetworkingMobileCertReply` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientNetworkingMobileCertReply {
    /// Field 1.
    pub encoded_cert: Option<String>,
}

impl Message for CMsgClientNetworkingMobileCertReply {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.encoded_cert = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.encoded_cert {
            encoder.write_string_field(1, value);
        }
    }
}

/// `CMsgClientGetAppOwnershipTicket` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetAppOwnershipTicket {
    /// Field 1.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientGetAppOwnershipTicket {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
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
    }
}

/// `CMsgClientGetAppOwnershipTicketResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetAppOwnershipTicketResponse {
    /// Field 1.
    pub eresult: Option<u32>,
    /// Field 2.
    pub app_id: Option<u32>,
    /// Field 3.
    pub ticket: Option<Vec<u8>>,
}

impl CMsgClientGetAppOwnershipTicketResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> u32 {
        self.eresult.unwrap_or(2_u32)
    }
}

impl Message for CMsgClientGetAppOwnershipTicketResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.ticket = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.eresult {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.ticket {
            encoder.write_bytes_field(3, value);
        }
    }
}

/// `CMsgClientSessionToken` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientSessionToken {
    /// Field 1.
    pub token: Option<u64>,
}

impl Message for CMsgClientSessionToken {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CMsgClientGameConnectTokens` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGameConnectTokens {
    /// Field 1.
    pub max_tokens_to_keep: Option<u32>,
    /// Field 2.
    pub tokens: Vec<Vec<u8>>,
}

impl CMsgClientGameConnectTokens {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn max_tokens_to_keep_or_default(&self) -> u32 {
        self.max_tokens_to_keep.unwrap_or(10_u32)
    }
}

impl Message for CMsgClientGameConnectTokens {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.max_tokens_to_keep = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.tokens.push(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.max_tokens_to_keep {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.tokens {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// Types nested inside [`CMsgClientGamesPlayed`].
pub mod c_msg_client_games_played {
    use super::*;

    /// `ProcessInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ProcessInfo {
        /// Field 1.
        pub process_id: Option<u32>,
        /// Field 2.
        pub process_id_parent: Option<u32>,
        /// Field 3.
        pub parent_is_steam: Option<bool>,
    }

    impl Message for ProcessInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.process_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.process_id_parent = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.parent_is_steam = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.process_id {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.process_id_parent {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.parent_is_steam {
                encoder.write_bool_field(3, *value);
            }
        }
    }

    /// `GamePlayed` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct GamePlayed {
        /// Field 1.
        pub steam_id_gs: Option<u64>,
        /// Field 2.
        pub game_id: Option<u64>,
        /// Field 3.
        pub deprecated_game_ip_address: Option<u32>,
        /// Field 4.
        pub game_port: Option<u32>,
        /// Field 5.
        pub is_secure: Option<bool>,
        /// Field 6.
        pub token: Option<Vec<u8>>,
        /// Field 7.
        pub game_extra_info: Option<String>,
        /// Field 8.
        pub game_data_blob: Option<Vec<u8>>,
        /// Field 9.
        pub process_id: Option<u32>,
        /// Field 10.
        pub streaming_provider_id: Option<u32>,
        /// Field 11.
        pub game_flags: Option<u32>,
        /// Field 12.
        pub owner_id: Option<u32>,
        /// Field 13.
        pub vr_hmd_vendor: Option<String>,
        /// Field 14.
        pub vr_hmd_model: Option<String>,
        /// Field 15.
        pub launch_option_type: Option<u32>,
        /// Field 16.
        pub primary_controller_type: Option<i32>,
        /// Field 17.
        pub primary_steam_controller_serial: Option<String>,
        /// Field 18.
        pub total_steam_controller_count: Option<u32>,
        /// Field 19.
        pub total_non_steam_controller_count: Option<u32>,
        /// Field 20.
        pub controller_workshop_file_id: Option<u64>,
        /// Field 21.
        pub launch_source: Option<u32>,
        /// Field 22.
        pub vr_hmd_runtime: Option<u32>,
        /// Field 23.
        pub game_ip_address: Option<crate::steammessages_base::CMsgIPAddress>,
        /// Field 24.
        pub controller_connection_type: Option<u32>,
        /// Field 25.
        pub game_os_platform: Option<i32>,
        /// Field 26.
        pub game_build_id: Option<u32>,
        /// Field 27.
        pub compat_tool_id: Option<u32>,
        /// Field 28.
        pub compat_tool_cmd: Option<String>,
        /// Field 29.
        pub compat_tool_build_id: Option<u32>,
        /// Field 30.
        pub beta_name: Option<String>,
        /// Field 31.
        pub dlc_context: Option<u32>,
        /// Field 32.
        pub process_id_list:
            Vec<crate::steammessages_clientserver::c_msg_client_games_played::ProcessInfo>,
    }

    impl GamePlayed {
        /// Field 15 , or its schema default when absent.
        #[must_use]
        pub fn launch_option_type_or_default(&self) -> u32 {
            self.launch_option_type.unwrap_or(0_u32)
        }
        /// Field 16 , or its schema default when absent.
        #[must_use]
        pub fn primary_controller_type_or_default(&self) -> i32 {
            self.primary_controller_type.unwrap_or(-1_i32)
        }
        /// Field 18 , or its schema default when absent.
        #[must_use]
        pub fn total_steam_controller_count_or_default(&self) -> u32 {
            self.total_steam_controller_count.unwrap_or(0_u32)
        }
        /// Field 19 , or its schema default when absent.
        #[must_use]
        pub fn total_non_steam_controller_count_or_default(&self) -> u32 {
            self.total_non_steam_controller_count.unwrap_or(0_u32)
        }
        /// Field 20 , or its schema default when absent.
        #[must_use]
        pub fn controller_workshop_file_id_or_default(&self) -> u64 {
            self.controller_workshop_file_id.unwrap_or(0_u64)
        }
        /// Field 21 , or its schema default when absent.
        #[must_use]
        pub fn launch_source_or_default(&self) -> u32 {
            self.launch_source.unwrap_or(0_u32)
        }
        /// Field 24 , or its schema default when absent.
        #[must_use]
        pub fn controller_connection_type_or_default(&self) -> u32 {
            self.controller_connection_type.unwrap_or(0_u32)
        }
        /// Field 27 , or its schema default when absent.
        #[must_use]
        pub fn compat_tool_id_or_default(&self) -> u32 {
            self.compat_tool_id.unwrap_or(0_u32)
        }
    }

    impl Message for GamePlayed {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.steam_id_gs = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.game_id = Some(decoder.read_fixed64()?);
                    }
                    3 => {
                        self.deprecated_game_ip_address = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.game_port = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.is_secure = Some(decoder.read_bool()?);
                    }
                    6 => {
                        self.token = Some(decoder.read_bytes()?.to_vec());
                    }
                    7 => {
                        self.game_extra_info = Some(decoder.read_string()?.to_owned());
                    }
                    8 => {
                        self.game_data_blob = Some(decoder.read_bytes()?.to_vec());
                    }
                    9 => {
                        self.process_id = Some(decoder.read_varint()? as u32);
                    }
                    10 => {
                        self.streaming_provider_id = Some(decoder.read_varint()? as u32);
                    }
                    11 => {
                        self.game_flags = Some(decoder.read_varint()? as u32);
                    }
                    12 => {
                        self.owner_id = Some(decoder.read_varint()? as u32);
                    }
                    13 => {
                        self.vr_hmd_vendor = Some(decoder.read_string()?.to_owned());
                    }
                    14 => {
                        self.vr_hmd_model = Some(decoder.read_string()?.to_owned());
                    }
                    15 => {
                        self.launch_option_type = Some(decoder.read_varint()? as u32);
                    }
                    16 => {
                        self.primary_controller_type = Some(decoder.read_varint()? as i32);
                    }
                    17 => {
                        self.primary_steam_controller_serial =
                            Some(decoder.read_string()?.to_owned());
                    }
                    18 => {
                        self.total_steam_controller_count = Some(decoder.read_varint()? as u32);
                    }
                    19 => {
                        self.total_non_steam_controller_count = Some(decoder.read_varint()? as u32);
                    }
                    20 => {
                        self.controller_workshop_file_id = Some(decoder.read_varint()?);
                    }
                    21 => {
                        self.launch_source = Some(decoder.read_varint()? as u32);
                    }
                    22 => {
                        self.vr_hmd_runtime = Some(decoder.read_varint()? as u32);
                    }
                    23 => {
                        self.game_ip_address = Some({
                            let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                            decoder.read_nested(|d| nested.merge(d))?;
                            nested
                        });
                    }
                    24 => {
                        self.controller_connection_type = Some(decoder.read_varint()? as u32);
                    }
                    25 => {
                        self.game_os_platform = Some(decoder.read_varint()? as i32);
                    }
                    26 => {
                        self.game_build_id = Some(decoder.read_varint()? as u32);
                    }
                    27 => {
                        self.compat_tool_id = Some(decoder.read_varint()? as u32);
                    }
                    28 => {
                        self.compat_tool_cmd = Some(decoder.read_string()?.to_owned());
                    }
                    29 => {
                        self.compat_tool_build_id = Some(decoder.read_varint()? as u32);
                    }
                    30 => {
                        self.beta_name = Some(decoder.read_string()?.to_owned());
                    }
                    31 => {
                        self.dlc_context = Some(decoder.read_varint()? as u32);
                    }
                    32 => {
                        self.process_id_list.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_games_played::ProcessInfo::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.steam_id_gs {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.game_id {
                encoder.write_fixed64_field(2, *value);
            }
            if let Some(value) = &self.deprecated_game_ip_address {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.game_port {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.is_secure {
                encoder.write_bool_field(5, *value);
            }
            if let Some(value) = &self.token {
                encoder.write_bytes_field(6, value);
            }
            if let Some(value) = &self.game_extra_info {
                encoder.write_string_field(7, value);
            }
            if let Some(value) = &self.game_data_blob {
                encoder.write_bytes_field(8, value);
            }
            if let Some(value) = &self.process_id {
                encoder.write_varint_field(9, u64::from(*value));
            }
            if let Some(value) = &self.streaming_provider_id {
                encoder.write_varint_field(10, u64::from(*value));
            }
            if let Some(value) = &self.game_flags {
                encoder.write_varint_field(11, u64::from(*value));
            }
            if let Some(value) = &self.owner_id {
                encoder.write_varint_field(12, u64::from(*value));
            }
            if let Some(value) = &self.vr_hmd_vendor {
                encoder.write_string_field(13, value);
            }
            if let Some(value) = &self.vr_hmd_model {
                encoder.write_string_field(14, value);
            }
            if let Some(value) = &self.launch_option_type {
                encoder.write_varint_field(15, u64::from(*value));
            }
            if let Some(value) = &self.primary_controller_type {
                encoder.write_int32_field(16, *value);
            }
            if let Some(value) = &self.primary_steam_controller_serial {
                encoder.write_string_field(17, value);
            }
            if let Some(value) = &self.total_steam_controller_count {
                encoder.write_varint_field(18, u64::from(*value));
            }
            if let Some(value) = &self.total_non_steam_controller_count {
                encoder.write_varint_field(19, u64::from(*value));
            }
            if let Some(value) = &self.controller_workshop_file_id {
                encoder.write_varint_field(20, *value);
            }
            if let Some(value) = &self.launch_source {
                encoder.write_varint_field(21, u64::from(*value));
            }
            if let Some(value) = &self.vr_hmd_runtime {
                encoder.write_varint_field(22, u64::from(*value));
            }
            if let Some(value) = &self.game_ip_address {
                encoder.write_message_field(23, value);
            }
            if let Some(value) = &self.controller_connection_type {
                encoder.write_varint_field(24, u64::from(*value));
            }
            if let Some(value) = &self.game_os_platform {
                encoder.write_int32_field(25, *value);
            }
            if let Some(value) = &self.game_build_id {
                encoder.write_varint_field(26, u64::from(*value));
            }
            if let Some(value) = &self.compat_tool_id {
                encoder.write_varint_field(27, u64::from(*value));
            }
            if let Some(value) = &self.compat_tool_cmd {
                encoder.write_string_field(28, value);
            }
            if let Some(value) = &self.compat_tool_build_id {
                encoder.write_varint_field(29, u64::from(*value));
            }
            if let Some(value) = &self.beta_name {
                encoder.write_string_field(30, value);
            }
            if let Some(value) = &self.dlc_context {
                encoder.write_varint_field(31, u64::from(*value));
            }
            for value in &self.process_id_list {
                encoder.write_message_field(32, value);
            }
        }
    }
}

/// `CMsgClientGamesPlayed` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGamesPlayed {
    /// Field 1.
    pub games_played: Vec<crate::steammessages_clientserver::c_msg_client_games_played::GamePlayed>,
    /// Field 2.
    pub client_os_type: Option<u32>,
    /// Field 3.
    pub cloud_gaming_platform: Option<u32>,
    /// Field 4.
    pub recent_reauthentication: Option<bool>,
}

impl Message for CMsgClientGamesPlayed {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.games_played.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_games_played::GamePlayed::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.client_os_type = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.cloud_gaming_platform = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.recent_reauthentication = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.games_played {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.client_os_type {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.cloud_gaming_platform {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.recent_reauthentication {
            encoder.write_bool_field(4, *value);
        }
    }
}

/// `CMsgGSApprove` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgGSApprove {
    /// Field 1.
    pub steam_id: Option<u64>,
    /// Field 2.
    pub owner_steam_id: Option<u64>,
}

impl Message for CMsgGSApprove {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.owner_steam_id = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.owner_steam_id {
            encoder.write_fixed64_field(2, *value);
        }
    }
}

/// `CMsgGSDeny` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgGSDeny {
    /// Field 1.
    pub steam_id: Option<u64>,
    /// Field 2.
    pub edeny_reason: Option<i32>,
    /// Field 3.
    pub deny_string: Option<String>,
}

impl Message for CMsgGSDeny {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.edeny_reason = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.deny_string = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.edeny_reason {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.deny_string {
            encoder.write_string_field(3, value);
        }
    }
}

/// `CMsgGSKick` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgGSKick {
    /// Field 1.
    pub steam_id: Option<u64>,
    /// Field 2.
    pub edeny_reason: Option<i32>,
}

impl Message for CMsgGSKick {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.edeny_reason = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.edeny_reason {
            encoder.write_int32_field(2, *value);
        }
    }
}

/// `CMsgClientAuthList` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAuthList {
    /// Field 1.
    pub tokens_left: Option<u32>,
    /// Field 2.
    pub last_request_seq: Option<u32>,
    /// Field 3.
    pub last_request_seq_from_server: Option<u32>,
    /// Field 4.
    pub tickets: Vec<crate::steammessages_base::CMsgAuthTicket>,
    /// Field 5.
    pub app_ids: Vec<u32>,
    /// Field 6.
    pub message_sequence: Option<u32>,
    /// Field 7.
    pub filtered: Option<bool>,
}

impl Message for CMsgClientAuthList {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.tokens_left = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.last_request_seq = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.last_request_seq_from_server = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.tickets.push({
                        let mut nested = crate::steammessages_base::CMsgAuthTicket::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                5 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.app_ids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                6 => {
                    self.message_sequence = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.filtered = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.tokens_left {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.last_request_seq {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.last_request_seq_from_server {
            encoder.write_varint_field(3, u64::from(*value));
        }
        for value in &self.tickets {
            encoder.write_message_field(4, value);
        }
        for value in &self.app_ids {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.message_sequence {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.filtered {
            encoder.write_bool_field(7, *value);
        }
    }
}

/// `CMsgClientAuthListAck` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAuthListAck {
    /// Field 1.
    pub ticket_crc: Vec<u32>,
    /// Field 2.
    pub app_ids: Vec<u32>,
    /// Field 3.
    pub message_sequence: Option<u32>,
}

impl Message for CMsgClientAuthListAck {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.ticket_crc,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.app_ids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                3 => {
                    self.message_sequence = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.ticket_crc {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.app_ids {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.message_sequence {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CMsgGameServerPolicyUpdate` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgGameServerPolicyUpdate {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub app_id_aux: Option<u32>,
}

impl Message for CMsgGameServerPolicyUpdate {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.app_id_aux = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.app_id_aux {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientLicenseList`].
pub mod c_msg_client_license_list {
    use super::*;

    /// `License` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct License {
        /// Field 1.
        pub package_id: Option<u32>,
        /// Field 2.
        pub time_created: Option<u32>,
        /// Field 3.
        pub time_next_process: Option<u32>,
        /// Field 4.
        pub minute_limit: Option<i32>,
        /// Field 5.
        pub minutes_used: Option<i32>,
        /// Field 6.
        pub payment_method: Option<u32>,
        /// Field 7.
        pub flags: Option<u32>,
        /// Field 8.
        pub purchase_country_code: Option<String>,
        /// Field 9.
        pub license_type: Option<u32>,
        /// Field 10.
        pub territory_code: Option<i32>,
        /// Field 11.
        pub change_number: Option<i32>,
        /// Field 12.
        pub owner_id: Option<u32>,
        /// Field 13.
        pub initial_period: Option<u32>,
        /// Field 14.
        pub initial_time_unit: Option<u32>,
        /// Field 15.
        pub renewal_period: Option<u32>,
        /// Field 16.
        pub renewal_time_unit: Option<u32>,
        /// Field 17.
        pub access_token: Option<u64>,
        /// Field 18.
        pub master_package_id: Option<u32>,
    }

    impl Message for License {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.package_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.time_created = Some(decoder.read_fixed32()?);
                    }
                    3 => {
                        self.time_next_process = Some(decoder.read_fixed32()?);
                    }
                    4 => {
                        self.minute_limit = Some(decoder.read_varint()? as i32);
                    }
                    5 => {
                        self.minutes_used = Some(decoder.read_varint()? as i32);
                    }
                    6 => {
                        self.payment_method = Some(decoder.read_varint()? as u32);
                    }
                    7 => {
                        self.flags = Some(decoder.read_varint()? as u32);
                    }
                    8 => {
                        self.purchase_country_code = Some(decoder.read_string()?.to_owned());
                    }
                    9 => {
                        self.license_type = Some(decoder.read_varint()? as u32);
                    }
                    10 => {
                        self.territory_code = Some(decoder.read_varint()? as i32);
                    }
                    11 => {
                        self.change_number = Some(decoder.read_varint()? as i32);
                    }
                    12 => {
                        self.owner_id = Some(decoder.read_varint()? as u32);
                    }
                    13 => {
                        self.initial_period = Some(decoder.read_varint()? as u32);
                    }
                    14 => {
                        self.initial_time_unit = Some(decoder.read_varint()? as u32);
                    }
                    15 => {
                        self.renewal_period = Some(decoder.read_varint()? as u32);
                    }
                    16 => {
                        self.renewal_time_unit = Some(decoder.read_varint()? as u32);
                    }
                    17 => {
                        self.access_token = Some(decoder.read_varint()?);
                    }
                    18 => {
                        self.master_package_id = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.package_id {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.time_created {
                encoder.write_fixed32_field(2, *value);
            }
            if let Some(value) = &self.time_next_process {
                encoder.write_fixed32_field(3, *value);
            }
            if let Some(value) = &self.minute_limit {
                encoder.write_int32_field(4, *value);
            }
            if let Some(value) = &self.minutes_used {
                encoder.write_int32_field(5, *value);
            }
            if let Some(value) = &self.payment_method {
                encoder.write_varint_field(6, u64::from(*value));
            }
            if let Some(value) = &self.flags {
                encoder.write_varint_field(7, u64::from(*value));
            }
            if let Some(value) = &self.purchase_country_code {
                encoder.write_string_field(8, value);
            }
            if let Some(value) = &self.license_type {
                encoder.write_varint_field(9, u64::from(*value));
            }
            if let Some(value) = &self.territory_code {
                encoder.write_int32_field(10, *value);
            }
            if let Some(value) = &self.change_number {
                encoder.write_int32_field(11, *value);
            }
            if let Some(value) = &self.owner_id {
                encoder.write_varint_field(12, u64::from(*value));
            }
            if let Some(value) = &self.initial_period {
                encoder.write_varint_field(13, u64::from(*value));
            }
            if let Some(value) = &self.initial_time_unit {
                encoder.write_varint_field(14, u64::from(*value));
            }
            if let Some(value) = &self.renewal_period {
                encoder.write_varint_field(15, u64::from(*value));
            }
            if let Some(value) = &self.renewal_time_unit {
                encoder.write_varint_field(16, u64::from(*value));
            }
            if let Some(value) = &self.access_token {
                encoder.write_varint_field(17, *value);
            }
            if let Some(value) = &self.master_package_id {
                encoder.write_varint_field(18, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientLicenseList` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientLicenseList {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub licenses: Vec<crate::steammessages_clientserver::c_msg_client_license_list::License>,
}

impl CMsgClientLicenseList {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientLicenseList {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.licenses.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_license_list::License::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        for value in &self.licenses {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgClientIsLimitedAccount` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientIsLimitedAccount {
    /// Field 1.
    pub bis_limited_account: Option<bool>,
    /// Field 2.
    pub bis_community_banned: Option<bool>,
    /// Field 3.
    pub bis_locked_account: Option<bool>,
    /// Field 4.
    pub bis_limited_account_allowed_to_invite_friends: Option<bool>,
}

impl Message for CMsgClientIsLimitedAccount {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.bis_limited_account = Some(decoder.read_bool()?);
                }
                2 => {
                    self.bis_community_banned = Some(decoder.read_bool()?);
                }
                3 => {
                    self.bis_locked_account = Some(decoder.read_bool()?);
                }
                4 => {
                    self.bis_limited_account_allowed_to_invite_friends = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.bis_limited_account {
            encoder.write_bool_field(1, *value);
        }
        if let Some(value) = &self.bis_community_banned {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.bis_locked_account {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.bis_limited_account_allowed_to_invite_friends {
            encoder.write_bool_field(4, *value);
        }
    }
}

/// Types nested inside [`CMsgClientRequestedClientStats`].
pub mod c_msg_client_requested_client_stats {
    use super::*;

    /// `StatsToSend` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct StatsToSend {
        /// Field 1.
        pub client_stat: Option<u32>,
        /// Field 2.
        pub stat_aggregate_method: Option<u32>,
    }

    impl Message for StatsToSend {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.client_stat = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.stat_aggregate_method = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.client_stat {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.stat_aggregate_method {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientRequestedClientStats` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestedClientStats {
    /// Field 1.
    pub stats_to_send:
        Vec<crate::steammessages_clientserver::c_msg_client_requested_client_stats::StatsToSend>,
}

impl Message for CMsgClientRequestedClientStats {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.stats_to_send.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_requested_client_stats::StatsToSend::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.stats_to_send {
            encoder.write_message_field(1, value);
        }
    }
}

/// Types nested inside [`CMsgClientStat2`].
pub mod c_msg_client_stat2 {
    use super::*;

    /// `StatDetail` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct StatDetail {
        /// Field 1.
        pub client_stat: Option<u32>,
        /// Field 2.
        pub ll_value: Option<i64>,
        /// Field 3.
        pub time_of_day: Option<u32>,
        /// Field 4.
        pub cell_id: Option<u32>,
        /// Field 5.
        pub depot_id: Option<u32>,
        /// Field 6.
        pub app_id: Option<u32>,
    }

    impl Message for StatDetail {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.client_stat = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.ll_value = Some(decoder.read_varint()? as i64);
                    }
                    3 => {
                        self.time_of_day = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.cell_id = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.depot_id = Some(decoder.read_varint()? as u32);
                    }
                    6 => {
                        self.app_id = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.client_stat {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.ll_value {
                encoder.write_varint_field(2, *value as u64);
            }
            if let Some(value) = &self.time_of_day {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.cell_id {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.depot_id {
                encoder.write_varint_field(5, u64::from(*value));
            }
            if let Some(value) = &self.app_id {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientStat2` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientStat2 {
    /// Field 1.
    pub stat_detail: Vec<crate::steammessages_clientserver::c_msg_client_stat2::StatDetail>,
}

impl Message for CMsgClientStat2 {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.stat_detail.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_stat2::StatDetail::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.stat_detail {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CMsgClientInviteToGame` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientInviteToGame {
    /// Field 1.
    pub steam_id_dest: Option<u64>,
    /// Field 2.
    pub steam_id_src: Option<u64>,
    /// Field 3.
    pub connect_string: Option<String>,
    /// Field 4.
    pub remote_play: Option<String>,
}

impl Message for CMsgClientInviteToGame {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id_dest = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.steam_id_src = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.connect_string = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.remote_play = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id_dest {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.steam_id_src {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.connect_string {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.remote_play {
            encoder.write_string_field(4, value);
        }
    }
}

/// `CMsgClientChatInvite` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientChatInvite {
    /// Field 1.
    pub steam_id_invited: Option<u64>,
    /// Field 2.
    pub steam_id_chat: Option<u64>,
    /// Field 3.
    pub steam_id_patron: Option<u64>,
    /// Field 4.
    pub chatroom_type: Option<i32>,
    /// Field 5.
    pub steam_id_friend_chat: Option<u64>,
    /// Field 6.
    pub chat_name: Option<String>,
    /// Field 7.
    pub game_id: Option<u64>,
}

impl Message for CMsgClientChatInvite {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id_invited = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.steam_id_chat = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.steam_id_patron = Some(decoder.read_fixed64()?);
                }
                4 => {
                    self.chatroom_type = Some(decoder.read_varint()? as i32);
                }
                5 => {
                    self.steam_id_friend_chat = Some(decoder.read_fixed64()?);
                }
                6 => {
                    self.chat_name = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.game_id = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steam_id_invited {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.steam_id_chat {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.steam_id_patron {
            encoder.write_fixed64_field(3, *value);
        }
        if let Some(value) = &self.chatroom_type {
            encoder.write_int32_field(4, *value);
        }
        if let Some(value) = &self.steam_id_friend_chat {
            encoder.write_fixed64_field(5, *value);
        }
        if let Some(value) = &self.chat_name {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.game_id {
            encoder.write_fixed64_field(7, *value);
        }
    }
}

/// Types nested inside [`CMsgClientConnectionStats`].
pub mod c_msg_client_connection_stats {
    use super::*;

    /// `Stats_Logon` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Stats_Logon {
        /// Field 1.
        pub connect_attempts: Option<i32>,
        /// Field 2.
        pub connect_successes: Option<i32>,
        /// Field 3.
        pub connect_failures: Option<i32>,
        /// Field 4.
        pub connections_dropped: Option<i32>,
        /// Field 5.
        pub seconds_running: Option<u32>,
        /// Field 6.
        pub msec_tologonthistime: Option<u32>,
        /// Field 7.
        pub count_bad_cms: Option<u32>,
        /// Field 8.
        pub no_udp_connectivity: Option<bool>,
        /// Field 9.
        pub no_tcp_connectivity: Option<bool>,
        /// Field 10.
        pub no_websocket_443_connectivity: Option<bool>,
        /// Field 11.
        pub no_websocket_non_443_connectivity: Option<bool>,
    }

    impl Message for Stats_Logon {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.connect_attempts = Some(decoder.read_varint()? as i32);
                    }
                    2 => {
                        self.connect_successes = Some(decoder.read_varint()? as i32);
                    }
                    3 => {
                        self.connect_failures = Some(decoder.read_varint()? as i32);
                    }
                    4 => {
                        self.connections_dropped = Some(decoder.read_varint()? as i32);
                    }
                    5 => {
                        self.seconds_running = Some(decoder.read_varint()? as u32);
                    }
                    6 => {
                        self.msec_tologonthistime = Some(decoder.read_varint()? as u32);
                    }
                    7 => {
                        self.count_bad_cms = Some(decoder.read_varint()? as u32);
                    }
                    8 => {
                        self.no_udp_connectivity = Some(decoder.read_bool()?);
                    }
                    9 => {
                        self.no_tcp_connectivity = Some(decoder.read_bool()?);
                    }
                    10 => {
                        self.no_websocket_443_connectivity = Some(decoder.read_bool()?);
                    }
                    11 => {
                        self.no_websocket_non_443_connectivity = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.connect_attempts {
                encoder.write_int32_field(1, *value);
            }
            if let Some(value) = &self.connect_successes {
                encoder.write_int32_field(2, *value);
            }
            if let Some(value) = &self.connect_failures {
                encoder.write_int32_field(3, *value);
            }
            if let Some(value) = &self.connections_dropped {
                encoder.write_int32_field(4, *value);
            }
            if let Some(value) = &self.seconds_running {
                encoder.write_varint_field(5, u64::from(*value));
            }
            if let Some(value) = &self.msec_tologonthistime {
                encoder.write_varint_field(6, u64::from(*value));
            }
            if let Some(value) = &self.count_bad_cms {
                encoder.write_varint_field(7, u64::from(*value));
            }
            if let Some(value) = &self.no_udp_connectivity {
                encoder.write_bool_field(8, *value);
            }
            if let Some(value) = &self.no_tcp_connectivity {
                encoder.write_bool_field(9, *value);
            }
            if let Some(value) = &self.no_websocket_443_connectivity {
                encoder.write_bool_field(10, *value);
            }
            if let Some(value) = &self.no_websocket_non_443_connectivity {
                encoder.write_bool_field(11, *value);
            }
        }
    }

    /// `Stats_UDP` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Stats_UDP {
        /// Field 1.
        pub pkts_sent: Option<u64>,
        /// Field 2.
        pub bytes_sent: Option<u64>,
        /// Field 3.
        pub pkts_recv: Option<u64>,
        /// Field 4.
        pub pkts_processed: Option<u64>,
        /// Field 5.
        pub bytes_recv: Option<u64>,
    }

    impl Message for Stats_UDP {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.pkts_sent = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.bytes_sent = Some(decoder.read_varint()?);
                    }
                    3 => {
                        self.pkts_recv = Some(decoder.read_varint()?);
                    }
                    4 => {
                        self.pkts_processed = Some(decoder.read_varint()?);
                    }
                    5 => {
                        self.bytes_recv = Some(decoder.read_varint()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.pkts_sent {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.bytes_sent {
                encoder.write_varint_field(2, *value);
            }
            if let Some(value) = &self.pkts_recv {
                encoder.write_varint_field(3, *value);
            }
            if let Some(value) = &self.pkts_processed {
                encoder.write_varint_field(4, *value);
            }
            if let Some(value) = &self.bytes_recv {
                encoder.write_varint_field(5, *value);
            }
        }
    }

    /// `Stats_VConn` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Stats_VConn {
        /// Field 1.
        pub connections_udp: Option<u32>,
        /// Field 2.
        pub connections_tcp: Option<u32>,
        /// Field 3.
        pub stats_udp:
            Option<crate::steammessages_clientserver::c_msg_client_connection_stats::Stats_UDP>,
        /// Field 4.
        pub pkts_abandoned: Option<u64>,
        /// Field 5.
        pub conn_req_received: Option<u64>,
        /// Field 6.
        pub pkts_resent: Option<u64>,
        /// Field 7.
        pub msgs_sent: Option<u64>,
        /// Field 8.
        pub msgs_sent_failed: Option<u64>,
        /// Field 9.
        pub msgs_recv: Option<u64>,
        /// Field 10.
        pub datagrams_sent: Option<u64>,
        /// Field 11.
        pub datagrams_recv: Option<u64>,
        /// Field 12.
        pub bad_pkts_recv: Option<u64>,
        /// Field 13.
        pub unknown_conn_pkts_recv: Option<u64>,
        /// Field 14.
        pub missed_pkts_recv: Option<u64>,
        /// Field 15.
        pub dup_pkts_recv: Option<u64>,
        /// Field 16.
        pub failed_connect_challenges: Option<u64>,
        /// Field 17.
        pub micro_sec_avg_latency: Option<u32>,
        /// Field 18.
        pub micro_sec_min_latency: Option<u32>,
        /// Field 19.
        pub micro_sec_max_latency: Option<u32>,
    }

    impl Message for Stats_VConn {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.connections_udp = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.connections_tcp = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.stats_udp = Some({
                            let mut nested = crate::steammessages_clientserver::c_msg_client_connection_stats::Stats_UDP::default();
                            decoder.read_nested(|d| nested.merge(d))?;
                            nested
                        });
                    }
                    4 => {
                        self.pkts_abandoned = Some(decoder.read_varint()?);
                    }
                    5 => {
                        self.conn_req_received = Some(decoder.read_varint()?);
                    }
                    6 => {
                        self.pkts_resent = Some(decoder.read_varint()?);
                    }
                    7 => {
                        self.msgs_sent = Some(decoder.read_varint()?);
                    }
                    8 => {
                        self.msgs_sent_failed = Some(decoder.read_varint()?);
                    }
                    9 => {
                        self.msgs_recv = Some(decoder.read_varint()?);
                    }
                    10 => {
                        self.datagrams_sent = Some(decoder.read_varint()?);
                    }
                    11 => {
                        self.datagrams_recv = Some(decoder.read_varint()?);
                    }
                    12 => {
                        self.bad_pkts_recv = Some(decoder.read_varint()?);
                    }
                    13 => {
                        self.unknown_conn_pkts_recv = Some(decoder.read_varint()?);
                    }
                    14 => {
                        self.missed_pkts_recv = Some(decoder.read_varint()?);
                    }
                    15 => {
                        self.dup_pkts_recv = Some(decoder.read_varint()?);
                    }
                    16 => {
                        self.failed_connect_challenges = Some(decoder.read_varint()?);
                    }
                    17 => {
                        self.micro_sec_avg_latency = Some(decoder.read_varint()? as u32);
                    }
                    18 => {
                        self.micro_sec_min_latency = Some(decoder.read_varint()? as u32);
                    }
                    19 => {
                        self.micro_sec_max_latency = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.connections_udp {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.connections_tcp {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.stats_udp {
                encoder.write_message_field(3, value);
            }
            if let Some(value) = &self.pkts_abandoned {
                encoder.write_varint_field(4, *value);
            }
            if let Some(value) = &self.conn_req_received {
                encoder.write_varint_field(5, *value);
            }
            if let Some(value) = &self.pkts_resent {
                encoder.write_varint_field(6, *value);
            }
            if let Some(value) = &self.msgs_sent {
                encoder.write_varint_field(7, *value);
            }
            if let Some(value) = &self.msgs_sent_failed {
                encoder.write_varint_field(8, *value);
            }
            if let Some(value) = &self.msgs_recv {
                encoder.write_varint_field(9, *value);
            }
            if let Some(value) = &self.datagrams_sent {
                encoder.write_varint_field(10, *value);
            }
            if let Some(value) = &self.datagrams_recv {
                encoder.write_varint_field(11, *value);
            }
            if let Some(value) = &self.bad_pkts_recv {
                encoder.write_varint_field(12, *value);
            }
            if let Some(value) = &self.unknown_conn_pkts_recv {
                encoder.write_varint_field(13, *value);
            }
            if let Some(value) = &self.missed_pkts_recv {
                encoder.write_varint_field(14, *value);
            }
            if let Some(value) = &self.dup_pkts_recv {
                encoder.write_varint_field(15, *value);
            }
            if let Some(value) = &self.failed_connect_challenges {
                encoder.write_varint_field(16, *value);
            }
            if let Some(value) = &self.micro_sec_avg_latency {
                encoder.write_varint_field(17, u64::from(*value));
            }
            if let Some(value) = &self.micro_sec_min_latency {
                encoder.write_varint_field(18, u64::from(*value));
            }
            if let Some(value) = &self.micro_sec_max_latency {
                encoder.write_varint_field(19, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientConnectionStats` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientConnectionStats {
    /// Field 1.
    pub stats_logon:
        Option<crate::steammessages_clientserver::c_msg_client_connection_stats::Stats_Logon>,
    /// Field 2.
    pub stats_vconn:
        Option<crate::steammessages_clientserver::c_msg_client_connection_stats::Stats_VConn>,
}

impl Message for CMsgClientConnectionStats {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.stats_logon = Some({
                        let mut nested = crate::steammessages_clientserver::c_msg_client_connection_stats::Stats_Logon::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                2 => {
                    self.stats_vconn = Some({
                        let mut nested = crate::steammessages_clientserver::c_msg_client_connection_stats::Stats_VConn::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.stats_logon {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.stats_vconn {
            encoder.write_message_field(2, value);
        }
    }
}

/// Types nested inside [`CMsgClientServersAvailable`].
pub mod c_msg_client_servers_available {
    use super::*;

    /// `Server_Types_Available` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Server_Types_Available {
        /// Field 1.
        pub server: Option<u32>,
        /// Field 2.
        pub changed: Option<bool>,
    }

    impl Message for Server_Types_Available {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.server = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.changed = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.server {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.changed {
                encoder.write_bool_field(2, *value);
            }
        }
    }
}

/// `CMsgClientServersAvailable` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServersAvailable {
    /// Field 1.
    pub server_types_available: Vec<
        crate::steammessages_clientserver::c_msg_client_servers_available::Server_Types_Available,
    >,
    /// Field 2.
    pub server_type_for_auth_services: Option<u32>,
}

impl Message for CMsgClientServersAvailable {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.server_types_available.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_servers_available::Server_Types_Available::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.server_type_for_auth_services = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.server_types_available {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.server_type_for_auth_services {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientReportOverlayDetourFailure` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientReportOverlayDetourFailure {
    /// Field 1.
    pub failure_strings: Vec<String>,
}

impl Message for CMsgClientReportOverlayDetourFailure {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.failure_strings.push(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.failure_strings {
            encoder.write_string_field(1, value);
        }
    }
}

/// `CMsgClientRequestEncryptedAppTicket` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestEncryptedAppTicket {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub userdata: Option<Vec<u8>>,
}

impl Message for CMsgClientRequestEncryptedAppTicket {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.userdata = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.userdata {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgClientRequestEncryptedAppTicketResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestEncryptedAppTicketResponse {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub eresult: Option<i32>,
    /// Field 3.
    pub encrypted_app_ticket: Option<crate::encrypted_app_ticket::EncryptedAppTicket>,
}

impl CMsgClientRequestEncryptedAppTicketResponse {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientRequestEncryptedAppTicketResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.encrypted_app_ticket = Some({
                        let mut nested = crate::encrypted_app_ticket::EncryptedAppTicket::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
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
        if let Some(value) = &self.eresult {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.encrypted_app_ticket {
            encoder.write_message_field(3, value);
        }
    }
}

/// `CMsgClientWalletInfoUpdate` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientWalletInfoUpdate {
    /// Field 1.
    pub has_wallet: Option<bool>,
    /// Field 2.
    pub balance: Option<i32>,
    /// Field 3.
    pub currency: Option<i32>,
    /// Field 4.
    pub balance_delayed: Option<i32>,
    /// Field 5.
    pub balance64: Option<i64>,
    /// Field 6.
    pub balance64_delayed: Option<i64>,
    /// Field 7.
    pub realm: Option<i32>,
}

impl Message for CMsgClientWalletInfoUpdate {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.has_wallet = Some(decoder.read_bool()?);
                }
                2 => {
                    self.balance = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.currency = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.balance_delayed = Some(decoder.read_varint()? as i32);
                }
                5 => {
                    self.balance64 = Some(decoder.read_varint()? as i64);
                }
                6 => {
                    self.balance64_delayed = Some(decoder.read_varint()? as i64);
                }
                7 => {
                    self.realm = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.has_wallet {
            encoder.write_bool_field(1, *value);
        }
        if let Some(value) = &self.balance {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.currency {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.balance_delayed {
            encoder.write_int32_field(4, *value);
        }
        if let Some(value) = &self.balance64 {
            encoder.write_varint_field(5, *value as u64);
        }
        if let Some(value) = &self.balance64_delayed {
            encoder.write_varint_field(6, *value as u64);
        }
        if let Some(value) = &self.realm {
            encoder.write_int32_field(7, *value);
        }
    }
}

/// `CMsgClientAMGetClanOfficers` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAMGetClanOfficers {
    /// Field 1.
    pub steamid_clan: Option<u64>,
}

impl Message for CMsgClientAMGetClanOfficers {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid_clan = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steamid_clan {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// `CMsgClientAMGetClanOfficersResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAMGetClanOfficersResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub steamid_clan: Option<u64>,
    /// Field 3.
    pub officer_count: Option<i32>,
}

impl CMsgClientAMGetClanOfficersResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientAMGetClanOfficersResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.steamid_clan = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.officer_count = Some(decoder.read_varint()? as i32);
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
        if let Some(value) = &self.steamid_clan {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.officer_count {
            encoder.write_int32_field(3, *value);
        }
    }
}

/// Types nested inside [`CMsgClientAMGetPersonaNameHistory`].
pub mod c_msg_client_am_get_persona_name_history {
    use super::*;

    /// `IdInstance` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct IdInstance {
        /// Field 1.
        pub steamid: Option<u64>,
    }

    impl Message for IdInstance {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.steamid = Some(decoder.read_fixed64()?);
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
        }
    }
}

/// `CMsgClientAMGetPersonaNameHistory` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAMGetPersonaNameHistory {
    /// Field 1.
    pub id_count: Option<i32>,
    /// Field 2.
    pub ids: Vec<
        crate::steammessages_clientserver::c_msg_client_am_get_persona_name_history::IdInstance,
    >,
}

impl Message for CMsgClientAMGetPersonaNameHistory {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.id_count = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.ids.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_am_get_persona_name_history::IdInstance::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.id_count {
            encoder.write_int32_field(1, *value);
        }
        for value in &self.ids {
            encoder.write_message_field(2, value);
        }
    }
}

/// Types nested inside [`CMsgClientAMGetPersonaNameHistoryResponse`].
pub mod c_msg_client_am_get_persona_name_history_response {
    use super::*;

    /// Types nested inside [`NameTableInstance`].
    pub mod name_table_instance {
        use super::*;

        /// `NameInstance` — generated from Valve's schema.
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct NameInstance {
            /// Field 1.
            pub name_since: Option<u32>,
            /// Field 2.
            pub name: Option<String>,
        }

        impl Message for NameInstance {
            fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
                while let Some(key) = decoder.read_key()? {
                    match key.number {
                        1 => {
                            self.name_since = Some(decoder.read_fixed32()?);
                        }
                        2 => {
                            self.name = Some(decoder.read_string()?.to_owned());
                        }
                        _ => decoder.skip_field(key.wire_type)?,
                    }
                }
                Ok(())
            }

            fn encode_raw(&self, encoder: &mut Encoder) {
                if let Some(value) = &self.name_since {
                    encoder.write_fixed32_field(1, *value);
                }
                if let Some(value) = &self.name {
                    encoder.write_string_field(2, value);
                }
            }
        }
    }

    /// `NameTableInstance` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct NameTableInstance {
        /// Field 1.
        pub eresult: Option<i32>,
        /// Field 2.
        pub steamid: Option<u64>,
        /// Field 3.
        pub names: Vec<crate::steammessages_clientserver::c_msg_client_am_get_persona_name_history_response::name_table_instance::NameInstance>,
    }

    impl NameTableInstance {
        /// Field 1 , or its schema default when absent.
        #[must_use]
        pub fn eresult_or_default(&self) -> i32 {
            self.eresult.unwrap_or(2_i32)
        }
    }

    impl Message for NameTableInstance {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.eresult = Some(decoder.read_varint()? as i32);
                    }
                    2 => {
                        self.steamid = Some(decoder.read_fixed64()?);
                    }
                    3 => {
                        self.names.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_am_get_persona_name_history_response::name_table_instance::NameInstance::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
            if let Some(value) = &self.steamid {
                encoder.write_fixed64_field(2, *value);
            }
            for value in &self.names {
                encoder.write_message_field(3, value);
            }
        }
    }
}

/// `CMsgClientAMGetPersonaNameHistoryResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAMGetPersonaNameHistoryResponse {
    /// Field 2.
    pub responses: Vec<crate::steammessages_clientserver::c_msg_client_am_get_persona_name_history_response::NameTableInstance>,
}

impl Message for CMsgClientAMGetPersonaNameHistoryResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                2 => {
                    self.responses.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_am_get_persona_name_history_response::NameTableInstance::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.responses {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgClientDeregisterWithServer` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDeregisterWithServer {
    /// Field 1.
    pub eservertype: Option<u32>,
    /// Field 2.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientDeregisterWithServer {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eservertype = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.eservertype {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientClanState`].
pub mod c_msg_client_clan_state {
    use super::*;

    /// `NameInfo` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct NameInfo {
        /// Field 1.
        pub clan_name: Option<String>,
        /// Field 2.
        pub sha_avatar: Option<Vec<u8>>,
    }

    impl Message for NameInfo {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.clan_name = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.sha_avatar = Some(decoder.read_bytes()?.to_vec());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.clan_name {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.sha_avatar {
                encoder.write_bytes_field(2, value);
            }
        }
    }

    /// `UserCounts` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct UserCounts {
        /// Field 1.
        pub members: Option<u32>,
        /// Field 2.
        pub online: Option<u32>,
        /// Field 3.
        pub chatting: Option<u32>,
        /// Field 4.
        pub in_game: Option<u32>,
        /// Field 5.
        pub chat_room_members: Option<u32>,
    }

    impl Message for UserCounts {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.members = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.online = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.chatting = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.in_game = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.chat_room_members = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.members {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.online {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.chatting {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.in_game {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.chat_room_members {
                encoder.write_varint_field(5, u64::from(*value));
            }
        }
    }

    /// `Event` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Event {
        /// Field 1.
        pub gid: Option<u64>,
        /// Field 2.
        pub event_time: Option<u32>,
        /// Field 3.
        pub headline: Option<String>,
        /// Field 4.
        pub game_id: Option<u64>,
        /// Field 5.
        pub just_posted: Option<bool>,
    }

    impl Message for Event {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.gid = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.event_time = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.headline = Some(decoder.read_string()?.to_owned());
                    }
                    4 => {
                        self.game_id = Some(decoder.read_fixed64()?);
                    }
                    5 => {
                        self.just_posted = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.gid {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.event_time {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.headline {
                encoder.write_string_field(3, value);
            }
            if let Some(value) = &self.game_id {
                encoder.write_fixed64_field(4, *value);
            }
            if let Some(value) = &self.just_posted {
                encoder.write_bool_field(5, *value);
            }
        }
    }
}

/// `CMsgClientClanState` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientClanState {
    /// Field 1.
    pub steamid_clan: Option<u64>,
    /// Field 3.
    pub clan_account_flags: Option<u32>,
    /// Field 4.
    pub name_info: Option<crate::steammessages_clientserver::c_msg_client_clan_state::NameInfo>,
    /// Field 5.
    pub user_counts: Option<crate::steammessages_clientserver::c_msg_client_clan_state::UserCounts>,
    /// Field 6.
    pub events: Vec<crate::steammessages_clientserver::c_msg_client_clan_state::Event>,
    /// Field 7.
    pub announcements: Vec<crate::steammessages_clientserver::c_msg_client_clan_state::Event>,
    /// Field 8.
    pub chat_room_private: Option<bool>,
}

impl Message for CMsgClientClanState {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid_clan = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.clan_account_flags = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.name_info = Some({
                        let mut nested = crate::steammessages_clientserver::c_msg_client_clan_state::NameInfo::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                5 => {
                    self.user_counts = Some({
                        let mut nested = crate::steammessages_clientserver::c_msg_client_clan_state::UserCounts::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                6 => {
                    self.events.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_clan_state::Event::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                7 => {
                    self.announcements.push({ let mut nested = crate::steammessages_clientserver::c_msg_client_clan_state::Event::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                8 => {
                    self.chat_room_private = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steamid_clan {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.clan_account_flags {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.name_info {
            encoder.write_message_field(4, value);
        }
        if let Some(value) = &self.user_counts {
            encoder.write_message_field(5, value);
        }
        for value in &self.events {
            encoder.write_message_field(6, value);
        }
        for value in &self.announcements {
            encoder.write_message_field(7, value);
        }
        if let Some(value) = &self.chat_room_private {
            encoder.write_bool_field(8, *value);
        }
    }
}
