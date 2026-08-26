//! Generated from `steammessages_clientserver_login.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `CMsgClientHeartBeat` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientHeartBeat {
    /// Field 1.
    pub send_reply: Option<bool>,
}

impl Message for CMsgClientHeartBeat {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.send_reply = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.send_reply {
            encoder.write_bool_field(1, *value);
        }
    }
}

/// `CMsgClientServerTimestampRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServerTimestampRequest {
    /// Field 1.
    pub client_request_timestamp: Option<u64>,
}

impl Message for CMsgClientServerTimestampRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_request_timestamp = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_request_timestamp {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CMsgClientServerTimestampResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServerTimestampResponse {
    /// Field 1.
    pub client_request_timestamp: Option<u64>,
    /// Field 2.
    pub server_timestamp_ms: Option<u64>,
}

impl Message for CMsgClientServerTimestampResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_request_timestamp = Some(decoder.read_varint()?);
                }
                2 => {
                    self.server_timestamp_ms = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_request_timestamp {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.server_timestamp_ms {
            encoder.write_varint_field(2, *value);
        }
    }
}

/// `CMsgClientSecret` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientSecret {
    /// Field 1.
    pub version: Option<u32>,
    /// Field 2.
    pub appid: Option<u32>,
    /// Field 3.
    pub deviceid: Option<u32>,
    /// Field 4.
    pub nonce: Option<u64>,
    /// Field 5.
    pub hmac: Option<Vec<u8>>,
}

impl Message for CMsgClientSecret {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.version = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.deviceid = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.nonce = Some(decoder.read_fixed64()?);
                }
                5 => {
                    self.hmac = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.version {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.deviceid {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.nonce {
            encoder.write_fixed64_field(4, *value);
        }
        if let Some(value) = &self.hmac {
            encoder.write_bytes_field(5, value);
        }
    }
}

/// `CMsgClientHello` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientHello {
    /// Field 1.
    pub protocol_version: Option<u32>,
}

impl Message for CMsgClientHello {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.protocol_version = Some(decoder.read_varint()? as u32);
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
    }
}

/// `CMsgClientLogon` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientLogon {
    /// Field 1.
    pub protocol_version: Option<u32>,
    /// Field 2.
    pub deprecated_obfustucated_private_ip: Option<u32>,
    /// Field 3.
    pub cell_id: Option<u32>,
    /// Field 4.
    pub last_session_id: Option<u32>,
    /// Field 5.
    pub client_package_version: Option<u32>,
    /// Field 6.
    pub client_language: Option<String>,
    /// Field 7.
    pub client_os_type: Option<u32>,
    /// Field 8.
    pub should_remember_password: Option<bool>,
    /// Field 9.
    pub wine_version: Option<String>,
    /// Field 10.
    pub deprecated_10: Option<u32>,
    /// Field 11.
    pub obfuscated_private_ip: Option<crate::steammessages_base::CMsgIPAddress>,
    /// Field 20.
    pub deprecated_public_ip: Option<u32>,
    /// Field 21.
    pub qos_level: Option<u32>,
    /// Field 22.
    pub client_supplied_steam_id: Option<u64>,
    /// Field 23.
    pub public_ip: Option<crate::steammessages_base::CMsgIPAddress>,
    /// Field 30.
    pub machine_id: Option<Vec<u8>>,
    /// Field 31.
    pub launcher_type: Option<u32>,
    /// Field 32.
    pub ui_mode: Option<u32>,
    /// Field 33.
    pub chat_mode: Option<u32>,
    /// Field 41.
    pub steam2_auth_ticket: Option<Vec<u8>>,
    /// Field 42.
    pub email_address: Option<String>,
    /// Field 43.
    pub rtime32_account_creation: Option<u32>,
    /// Field 50.
    pub account_name: Option<String>,
    /// Field 51.
    pub password: Option<String>,
    /// Field 52.
    pub game_server_token: Option<String>,
    /// Field 60.
    pub login_key: Option<String>,
    /// Field 70.
    pub was_converted_deprecated_msg: Option<bool>,
    /// Field 80.
    pub anon_user_target_account_name: Option<String>,
    /// Field 81.
    pub resolved_user_steam_id: Option<u64>,
    /// Field 82.
    pub eresult_sentryfile: Option<i32>,
    /// Field 83.
    pub sha_sentryfile: Option<Vec<u8>>,
    /// Field 84.
    pub auth_code: Option<String>,
    /// Field 85.
    pub otp_type: Option<i32>,
    /// Field 86.
    pub otp_value: Option<u32>,
    /// Field 87.
    pub otp_identifier: Option<String>,
    /// Field 88.
    pub steam2_ticket_request: Option<bool>,
    /// Field 90.
    pub sony_psn_ticket: Option<Vec<u8>>,
    /// Field 91.
    pub sony_psn_service_id: Option<String>,
    /// Field 92.
    pub create_new_psn_linked_account_if_needed: Option<bool>,
    /// Field 93.
    pub sony_psn_name: Option<String>,
    /// Field 94.
    pub game_server_app_id: Option<i32>,
    /// Field 95.
    pub steamguard_dont_remember_computer: Option<bool>,
    /// Field 96.
    pub machine_name: Option<String>,
    /// Field 97.
    pub machine_name_userchosen: Option<String>,
    /// Field 98.
    pub country_override: Option<String>,
    /// Field 100.
    pub client_instance_id: Option<u64>,
    /// Field 101.
    pub two_factor_code: Option<String>,
    /// Field 102.
    pub supports_rate_limit_response: Option<bool>,
    /// Field 103.
    pub web_logon_nonce: Option<String>,
    /// Field 104.
    pub priority_reason: Option<i32>,
    /// Field 105.
    pub embedded_client_secret: Option<crate::steammessages_clientserver_login::CMsgClientSecret>,
    /// Field 106.
    pub disable_partner_autogrants: Option<bool>,
    /// Field 108.
    pub access_token: Option<String>,
    /// Field 109.
    pub is_chrome_os: Option<bool>,
    /// Field 111.
    pub gaming_device_type: Option<u32>,
}

impl CMsgClientLogon {
    /// Field 8 , or its schema default when absent.
    #[must_use]
    pub fn should_remember_password_or_default(&self) -> bool {
        self.should_remember_password.unwrap_or(false)
    }
    /// Field 31 , or its schema default when absent.
    #[must_use]
    pub fn launcher_type_or_default(&self) -> u32 {
        self.launcher_type.unwrap_or(0_u32)
    }
    /// Field 32 , or its schema default when absent.
    #[must_use]
    pub fn ui_mode_or_default(&self) -> u32 {
        self.ui_mode.unwrap_or(0_u32)
    }
    /// Field 33 , or its schema default when absent.
    #[must_use]
    pub fn chat_mode_or_default(&self) -> u32 {
        self.chat_mode.unwrap_or(0_u32)
    }
    /// Field 70 , or its schema default when absent.
    #[must_use]
    pub fn was_converted_deprecated_msg_or_default(&self) -> bool {
        self.was_converted_deprecated_msg.unwrap_or(false)
    }
    /// Field 92 , or its schema default when absent.
    #[must_use]
    pub fn create_new_psn_linked_account_if_needed_or_default(&self) -> bool {
        self.create_new_psn_linked_account_if_needed
            .unwrap_or(false)
    }
}

impl Message for CMsgClientLogon {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.protocol_version = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.deprecated_obfustucated_private_ip = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.cell_id = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.last_session_id = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.client_package_version = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.client_language = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.client_os_type = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.should_remember_password = Some(decoder.read_bool()?);
                }
                9 => {
                    self.wine_version = Some(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.deprecated_10 = Some(decoder.read_varint()? as u32);
                }
                11 => {
                    self.obfuscated_private_ip = Some({
                        let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                20 => {
                    self.deprecated_public_ip = Some(decoder.read_varint()? as u32);
                }
                21 => {
                    self.qos_level = Some(decoder.read_varint()? as u32);
                }
                22 => {
                    self.client_supplied_steam_id = Some(decoder.read_fixed64()?);
                }
                23 => {
                    self.public_ip = Some({
                        let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                30 => {
                    self.machine_id = Some(decoder.read_bytes()?.to_vec());
                }
                31 => {
                    self.launcher_type = Some(decoder.read_varint()? as u32);
                }
                32 => {
                    self.ui_mode = Some(decoder.read_varint()? as u32);
                }
                33 => {
                    self.chat_mode = Some(decoder.read_varint()? as u32);
                }
                41 => {
                    self.steam2_auth_ticket = Some(decoder.read_bytes()?.to_vec());
                }
                42 => {
                    self.email_address = Some(decoder.read_string()?.to_owned());
                }
                43 => {
                    self.rtime32_account_creation = Some(decoder.read_fixed32()?);
                }
                50 => {
                    self.account_name = Some(decoder.read_string()?.to_owned());
                }
                51 => {
                    self.password = Some(decoder.read_string()?.to_owned());
                }
                52 => {
                    self.game_server_token = Some(decoder.read_string()?.to_owned());
                }
                60 => {
                    self.login_key = Some(decoder.read_string()?.to_owned());
                }
                70 => {
                    self.was_converted_deprecated_msg = Some(decoder.read_bool()?);
                }
                80 => {
                    self.anon_user_target_account_name = Some(decoder.read_string()?.to_owned());
                }
                81 => {
                    self.resolved_user_steam_id = Some(decoder.read_fixed64()?);
                }
                82 => {
                    self.eresult_sentryfile = Some(decoder.read_varint()? as i32);
                }
                83 => {
                    self.sha_sentryfile = Some(decoder.read_bytes()?.to_vec());
                }
                84 => {
                    self.auth_code = Some(decoder.read_string()?.to_owned());
                }
                85 => {
                    self.otp_type = Some(decoder.read_varint()? as i32);
                }
                86 => {
                    self.otp_value = Some(decoder.read_varint()? as u32);
                }
                87 => {
                    self.otp_identifier = Some(decoder.read_string()?.to_owned());
                }
                88 => {
                    self.steam2_ticket_request = Some(decoder.read_bool()?);
                }
                90 => {
                    self.sony_psn_ticket = Some(decoder.read_bytes()?.to_vec());
                }
                91 => {
                    self.sony_psn_service_id = Some(decoder.read_string()?.to_owned());
                }
                92 => {
                    self.create_new_psn_linked_account_if_needed = Some(decoder.read_bool()?);
                }
                93 => {
                    self.sony_psn_name = Some(decoder.read_string()?.to_owned());
                }
                94 => {
                    self.game_server_app_id = Some(decoder.read_varint()? as i32);
                }
                95 => {
                    self.steamguard_dont_remember_computer = Some(decoder.read_bool()?);
                }
                96 => {
                    self.machine_name = Some(decoder.read_string()?.to_owned());
                }
                97 => {
                    self.machine_name_userchosen = Some(decoder.read_string()?.to_owned());
                }
                98 => {
                    self.country_override = Some(decoder.read_string()?.to_owned());
                }
                100 => {
                    self.client_instance_id = Some(decoder.read_varint()?);
                }
                101 => {
                    self.two_factor_code = Some(decoder.read_string()?.to_owned());
                }
                102 => {
                    self.supports_rate_limit_response = Some(decoder.read_bool()?);
                }
                103 => {
                    self.web_logon_nonce = Some(decoder.read_string()?.to_owned());
                }
                104 => {
                    self.priority_reason = Some(decoder.read_varint()? as i32);
                }
                105 => {
                    self.embedded_client_secret = Some({
                        let mut nested =
                            crate::steammessages_clientserver_login::CMsgClientSecret::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                106 => {
                    self.disable_partner_autogrants = Some(decoder.read_bool()?);
                }
                108 => {
                    self.access_token = Some(decoder.read_string()?.to_owned());
                }
                109 => {
                    self.is_chrome_os = Some(decoder.read_bool()?);
                }
                111 => {
                    self.gaming_device_type = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.deprecated_obfustucated_private_ip {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.cell_id {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.last_session_id {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.client_package_version {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.client_language {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.client_os_type {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.should_remember_password {
            encoder.write_bool_field(8, *value);
        }
        if let Some(value) = &self.wine_version {
            encoder.write_string_field(9, value);
        }
        if let Some(value) = &self.deprecated_10 {
            encoder.write_varint_field(10, u64::from(*value));
        }
        if let Some(value) = &self.obfuscated_private_ip {
            encoder.write_message_field(11, value);
        }
        if let Some(value) = &self.deprecated_public_ip {
            encoder.write_varint_field(20, u64::from(*value));
        }
        if let Some(value) = &self.qos_level {
            encoder.write_varint_field(21, u64::from(*value));
        }
        if let Some(value) = &self.client_supplied_steam_id {
            encoder.write_fixed64_field(22, *value);
        }
        if let Some(value) = &self.public_ip {
            encoder.write_message_field(23, value);
        }
        if let Some(value) = &self.machine_id {
            encoder.write_bytes_field(30, value);
        }
        if let Some(value) = &self.launcher_type {
            encoder.write_varint_field(31, u64::from(*value));
        }
        if let Some(value) = &self.ui_mode {
            encoder.write_varint_field(32, u64::from(*value));
        }
        if let Some(value) = &self.chat_mode {
            encoder.write_varint_field(33, u64::from(*value));
        }
        if let Some(value) = &self.steam2_auth_ticket {
            encoder.write_bytes_field(41, value);
        }
        if let Some(value) = &self.email_address {
            encoder.write_string_field(42, value);
        }
        if let Some(value) = &self.rtime32_account_creation {
            encoder.write_fixed32_field(43, *value);
        }
        if let Some(value) = &self.account_name {
            encoder.write_string_field(50, value);
        }
        if let Some(value) = &self.password {
            encoder.write_string_field(51, value);
        }
        if let Some(value) = &self.game_server_token {
            encoder.write_string_field(52, value);
        }
        if let Some(value) = &self.login_key {
            encoder.write_string_field(60, value);
        }
        if let Some(value) = &self.was_converted_deprecated_msg {
            encoder.write_bool_field(70, *value);
        }
        if let Some(value) = &self.anon_user_target_account_name {
            encoder.write_string_field(80, value);
        }
        if let Some(value) = &self.resolved_user_steam_id {
            encoder.write_fixed64_field(81, *value);
        }
        if let Some(value) = &self.eresult_sentryfile {
            encoder.write_int32_field(82, *value);
        }
        if let Some(value) = &self.sha_sentryfile {
            encoder.write_bytes_field(83, value);
        }
        if let Some(value) = &self.auth_code {
            encoder.write_string_field(84, value);
        }
        if let Some(value) = &self.otp_type {
            encoder.write_int32_field(85, *value);
        }
        if let Some(value) = &self.otp_value {
            encoder.write_varint_field(86, u64::from(*value));
        }
        if let Some(value) = &self.otp_identifier {
            encoder.write_string_field(87, value);
        }
        if let Some(value) = &self.steam2_ticket_request {
            encoder.write_bool_field(88, *value);
        }
        if let Some(value) = &self.sony_psn_ticket {
            encoder.write_bytes_field(90, value);
        }
        if let Some(value) = &self.sony_psn_service_id {
            encoder.write_string_field(91, value);
        }
        if let Some(value) = &self.create_new_psn_linked_account_if_needed {
            encoder.write_bool_field(92, *value);
        }
        if let Some(value) = &self.sony_psn_name {
            encoder.write_string_field(93, value);
        }
        if let Some(value) = &self.game_server_app_id {
            encoder.write_int32_field(94, *value);
        }
        if let Some(value) = &self.steamguard_dont_remember_computer {
            encoder.write_bool_field(95, *value);
        }
        if let Some(value) = &self.machine_name {
            encoder.write_string_field(96, value);
        }
        if let Some(value) = &self.machine_name_userchosen {
            encoder.write_string_field(97, value);
        }
        if let Some(value) = &self.country_override {
            encoder.write_string_field(98, value);
        }
        if let Some(value) = &self.client_instance_id {
            encoder.write_varint_field(100, *value);
        }
        if let Some(value) = &self.two_factor_code {
            encoder.write_string_field(101, value);
        }
        if let Some(value) = &self.supports_rate_limit_response {
            encoder.write_bool_field(102, *value);
        }
        if let Some(value) = &self.web_logon_nonce {
            encoder.write_string_field(103, value);
        }
        if let Some(value) = &self.priority_reason {
            encoder.write_int32_field(104, *value);
        }
        if let Some(value) = &self.embedded_client_secret {
            encoder.write_message_field(105, value);
        }
        if let Some(value) = &self.disable_partner_autogrants {
            encoder.write_bool_field(106, *value);
        }
        if let Some(value) = &self.access_token {
            encoder.write_string_field(108, value);
        }
        if let Some(value) = &self.is_chrome_os {
            encoder.write_bool_field(109, *value);
        }
        if let Some(value) = &self.gaming_device_type {
            encoder.write_varint_field(111, u64::from(*value));
        }
    }
}

/// `CMsgClientLogonResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientLogonResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub legacy_out_of_game_heartbeat_seconds: Option<i32>,
    /// Field 3.
    pub heartbeat_seconds: Option<i32>,
    /// Field 4.
    pub deprecated_public_ip: Option<u32>,
    /// Field 5.
    pub rtime32_server_time: Option<u32>,
    /// Field 6.
    pub account_flags: Option<u32>,
    /// Field 7.
    pub cell_id: Option<u32>,
    /// Field 8.
    pub email_domain: Option<String>,
    /// Field 9.
    pub steam2_ticket: Option<Vec<u8>>,
    /// Field 10.
    pub eresult_extended: Option<i32>,
    /// Field 12.
    pub cell_id_ping_threshold: Option<u32>,
    /// Field 13.
    pub deprecated_use_pics: Option<bool>,
    /// Field 14.
    pub vanity_url: Option<String>,
    /// Field 15.
    pub public_ip: Option<crate::steammessages_base::CMsgIPAddress>,
    /// Field 16.
    pub user_country: Option<String>,
    /// Field 20.
    pub client_supplied_steamid: Option<u64>,
    /// Field 21.
    pub ip_country_code: Option<String>,
    /// Field 22.
    pub parental_settings: Option<Vec<u8>>,
    /// Field 23.
    pub parental_setting_signature: Option<Vec<u8>>,
    /// Field 24.
    pub count_loginfailures_to_migrate: Option<i32>,
    /// Field 25.
    pub count_disconnects_to_migrate: Option<i32>,
    /// Field 26.
    pub ogs_data_report_time_window: Option<i32>,
    /// Field 27.
    pub client_instance_id: Option<u64>,
    /// Field 28.
    pub force_client_update_check: Option<bool>,
    /// Field 29.
    pub agreement_session_url: Option<String>,
    /// Field 30.
    pub token_id: Option<u64>,
    /// Field 31.
    pub family_group_id: Option<u64>,
}

impl CMsgClientLogonResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientLogonResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.legacy_out_of_game_heartbeat_seconds = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.heartbeat_seconds = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.deprecated_public_ip = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.rtime32_server_time = Some(decoder.read_fixed32()?);
                }
                6 => {
                    self.account_flags = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.cell_id = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.email_domain = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.steam2_ticket = Some(decoder.read_bytes()?.to_vec());
                }
                10 => {
                    self.eresult_extended = Some(decoder.read_varint()? as i32);
                }
                12 => {
                    self.cell_id_ping_threshold = Some(decoder.read_varint()? as u32);
                }
                13 => {
                    self.deprecated_use_pics = Some(decoder.read_bool()?);
                }
                14 => {
                    self.vanity_url = Some(decoder.read_string()?.to_owned());
                }
                15 => {
                    self.public_ip = Some({
                        let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                16 => {
                    self.user_country = Some(decoder.read_string()?.to_owned());
                }
                20 => {
                    self.client_supplied_steamid = Some(decoder.read_fixed64()?);
                }
                21 => {
                    self.ip_country_code = Some(decoder.read_string()?.to_owned());
                }
                22 => {
                    self.parental_settings = Some(decoder.read_bytes()?.to_vec());
                }
                23 => {
                    self.parental_setting_signature = Some(decoder.read_bytes()?.to_vec());
                }
                24 => {
                    self.count_loginfailures_to_migrate = Some(decoder.read_varint()? as i32);
                }
                25 => {
                    self.count_disconnects_to_migrate = Some(decoder.read_varint()? as i32);
                }
                26 => {
                    self.ogs_data_report_time_window = Some(decoder.read_varint()? as i32);
                }
                27 => {
                    self.client_instance_id = Some(decoder.read_varint()?);
                }
                28 => {
                    self.force_client_update_check = Some(decoder.read_bool()?);
                }
                29 => {
                    self.agreement_session_url = Some(decoder.read_string()?.to_owned());
                }
                30 => {
                    self.token_id = Some(decoder.read_varint()?);
                }
                31 => {
                    self.family_group_id = Some(decoder.read_varint()?);
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
        if let Some(value) = &self.legacy_out_of_game_heartbeat_seconds {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.heartbeat_seconds {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.deprecated_public_ip {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.rtime32_server_time {
            encoder.write_fixed32_field(5, *value);
        }
        if let Some(value) = &self.account_flags {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.cell_id {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.email_domain {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.steam2_ticket {
            encoder.write_bytes_field(9, value);
        }
        if let Some(value) = &self.eresult_extended {
            encoder.write_int32_field(10, *value);
        }
        if let Some(value) = &self.cell_id_ping_threshold {
            encoder.write_varint_field(12, u64::from(*value));
        }
        if let Some(value) = &self.deprecated_use_pics {
            encoder.write_bool_field(13, *value);
        }
        if let Some(value) = &self.vanity_url {
            encoder.write_string_field(14, value);
        }
        if let Some(value) = &self.public_ip {
            encoder.write_message_field(15, value);
        }
        if let Some(value) = &self.user_country {
            encoder.write_string_field(16, value);
        }
        if let Some(value) = &self.client_supplied_steamid {
            encoder.write_fixed64_field(20, *value);
        }
        if let Some(value) = &self.ip_country_code {
            encoder.write_string_field(21, value);
        }
        if let Some(value) = &self.parental_settings {
            encoder.write_bytes_field(22, value);
        }
        if let Some(value) = &self.parental_setting_signature {
            encoder.write_bytes_field(23, value);
        }
        if let Some(value) = &self.count_loginfailures_to_migrate {
            encoder.write_int32_field(24, *value);
        }
        if let Some(value) = &self.count_disconnects_to_migrate {
            encoder.write_int32_field(25, *value);
        }
        if let Some(value) = &self.ogs_data_report_time_window {
            encoder.write_int32_field(26, *value);
        }
        if let Some(value) = &self.client_instance_id {
            encoder.write_varint_field(27, *value);
        }
        if let Some(value) = &self.force_client_update_check {
            encoder.write_bool_field(28, *value);
        }
        if let Some(value) = &self.agreement_session_url {
            encoder.write_string_field(29, value);
        }
        if let Some(value) = &self.token_id {
            encoder.write_varint_field(30, *value);
        }
        if let Some(value) = &self.family_group_id {
            encoder.write_varint_field(31, *value);
        }
    }
}

/// `CMsgClientRequestWebAPIAuthenticateUserNonce` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestWebAPIAuthenticateUserNonce {
    /// Field 1.
    pub token_type: Option<i32>,
}

impl CMsgClientRequestWebAPIAuthenticateUserNonce {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn token_type_or_default(&self) -> i32 {
        self.token_type.unwrap_or(-1_i32)
    }
}

impl Message for CMsgClientRequestWebAPIAuthenticateUserNonce {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token_type = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token_type {
            encoder.write_int32_field(1, *value);
        }
    }
}

/// `CMsgClientRequestWebAPIAuthenticateUserNonceResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestWebAPIAuthenticateUserNonceResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 11.
    pub webapi_authenticate_user_nonce: Option<String>,
    /// Field 3.
    pub token_type: Option<i32>,
}

impl CMsgClientRequestWebAPIAuthenticateUserNonceResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
    /// Field 3 , or its schema default when absent.
    #[must_use]
    pub fn token_type_or_default(&self) -> i32 {
        self.token_type.unwrap_or(-1_i32)
    }
}

impl Message for CMsgClientRequestWebAPIAuthenticateUserNonceResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                11 => {
                    self.webapi_authenticate_user_nonce = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.token_type = Some(decoder.read_varint()? as i32);
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
        if let Some(value) = &self.webapi_authenticate_user_nonce {
            encoder.write_string_field(11, value);
        }
        if let Some(value) = &self.token_type {
            encoder.write_int32_field(3, *value);
        }
    }
}

/// `CMsgClientLogOff` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientLogOff {}

impl Message for CMsgClientLogOff {
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

/// `CMsgClientLoggedOff` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientLoggedOff {
    /// Field 1.
    pub eresult: Option<i32>,
}

impl CMsgClientLoggedOff {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientLoggedOff {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
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
    }
}

/// `CMsgClientNewLoginKey` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientNewLoginKey {
    /// Field 1.
    pub unique_id: Option<u32>,
    /// Field 2.
    pub login_key: Option<String>,
}

impl Message for CMsgClientNewLoginKey {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.unique_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.login_key = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.unique_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.login_key {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CMsgClientNewLoginKeyAccepted` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientNewLoginKeyAccepted {
    /// Field 1.
    pub unique_id: Option<u32>,
}

impl Message for CMsgClientNewLoginKeyAccepted {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.unique_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.unique_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
    }
}

/// `CMsgClientAccountInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAccountInfo {
    /// Field 1.
    pub persona_name: Option<String>,
    /// Field 2.
    pub ip_country: Option<String>,
    /// Field 5.
    pub count_authed_computers: Option<i32>,
    /// Field 7.
    pub account_flags: Option<u32>,
    /// Field 15.
    pub steamguard_machine_name_user_chosen: Option<String>,
    /// Field 16.
    pub is_phone_verified: Option<bool>,
    /// Field 17.
    pub two_factor_state: Option<u32>,
    /// Field 18.
    pub is_phone_identifying: Option<bool>,
    /// Field 19.
    pub is_phone_needing_reverify: Option<bool>,
}

impl Message for CMsgClientAccountInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.persona_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.ip_country = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.count_authed_computers = Some(decoder.read_varint()? as i32);
                }
                7 => {
                    self.account_flags = Some(decoder.read_varint()? as u32);
                }
                15 => {
                    self.steamguard_machine_name_user_chosen =
                        Some(decoder.read_string()?.to_owned());
                }
                16 => {
                    self.is_phone_verified = Some(decoder.read_bool()?);
                }
                17 => {
                    self.two_factor_state = Some(decoder.read_varint()? as u32);
                }
                18 => {
                    self.is_phone_identifying = Some(decoder.read_bool()?);
                }
                19 => {
                    self.is_phone_needing_reverify = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.persona_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.ip_country {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.count_authed_computers {
            encoder.write_int32_field(5, *value);
        }
        if let Some(value) = &self.account_flags {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.steamguard_machine_name_user_chosen {
            encoder.write_string_field(15, value);
        }
        if let Some(value) = &self.is_phone_verified {
            encoder.write_bool_field(16, *value);
        }
        if let Some(value) = &self.two_factor_state {
            encoder.write_varint_field(17, u64::from(*value));
        }
        if let Some(value) = &self.is_phone_identifying {
            encoder.write_bool_field(18, *value);
        }
        if let Some(value) = &self.is_phone_needing_reverify {
            encoder.write_bool_field(19, *value);
        }
    }
}

/// `CMsgClientChallengeRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientChallengeRequest {
    /// Field 1.
    pub steamid: Option<u64>,
}

impl Message for CMsgClientChallengeRequest {
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

/// `CMsgClientChallengeResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientChallengeResponse {
    /// Field 1.
    pub challenge: Option<u64>,
}

impl Message for CMsgClientChallengeResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.challenge = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.challenge {
            encoder.write_fixed64_field(1, *value);
        }
    }
}
