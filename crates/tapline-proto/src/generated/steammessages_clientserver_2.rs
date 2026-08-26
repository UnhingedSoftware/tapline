//! Generated from `steammessages_clientserver_2.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `CMsgClientUpdateUserGameInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientUpdateUserGameInfo {
    /// Field 1.
    pub steamid_idgs: Option<u64>,
    /// Field 2.
    pub gameid: Option<u64>,
    /// Field 3.
    pub game_ip: Option<u32>,
    /// Field 4.
    pub game_port: Option<u32>,
    /// Field 5.
    pub token: Option<Vec<u8>>,
}

impl Message for CMsgClientUpdateUserGameInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid_idgs = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.gameid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.game_ip = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.game_port = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.token = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.steamid_idgs {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.gameid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.game_ip {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.game_port {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.token {
            encoder.write_bytes_field(5, value);
        }
    }
}

/// `CMsgClientRichPresenceUpload` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRichPresenceUpload {
    /// Field 1.
    pub rich_presence_kv: Option<Vec<u8>>,
    /// Field 2.
    pub steamid_broadcast: Vec<u64>,
}

impl Message for CMsgClientRichPresenceUpload {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.rich_presence_kv = Some(decoder.read_bytes()?.to_vec());
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.steamid_broadcast,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.rich_presence_kv {
            encoder.write_bytes_field(1, value);
        }
        for value in &self.steamid_broadcast {
            encoder.write_fixed64_field(2, *value);
        }
    }
}

/// `CMsgClientRichPresenceRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRichPresenceRequest {
    /// Field 1.
    pub steamid_request: Vec<u64>,
}

impl Message for CMsgClientRichPresenceRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.steamid_request,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.steamid_request {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// Types nested inside [`CMsgClientRichPresenceInfo`].
pub mod c_msg_client_rich_presence_info {
    use super::*;

    /// `KV` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct KV {
        /// Field 1.
        pub key: Option<String>,
        /// Field 2.
        pub value: Option<String>,
    }

    impl Message for KV {
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

    /// `RichPresence` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct RichPresence {
        /// Field 1.
        pub steamid_user: Option<u64>,
        /// Field 3.
        pub rich_presense:
            Vec<crate::steammessages_clientserver_2::c_msg_client_rich_presence_info::KV>,
    }

    impl Message for RichPresence {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.steamid_user = Some(decoder.read_fixed64()?);
                    }
                    3 => {
                        self.rich_presense.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_rich_presence_info::KV::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.steamid_user {
                encoder.write_fixed64_field(1, *value);
            }
            for value in &self.rich_presense {
                encoder.write_message_field(3, value);
            }
        }
    }
}

/// `CMsgClientRichPresenceInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRichPresenceInfo {
    /// Field 1.
    pub rich_presence:
        Vec<crate::steammessages_clientserver_2::c_msg_client_rich_presence_info::RichPresence>,
}

impl Message for CMsgClientRichPresenceInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.rich_presence.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_rich_presence_info::RichPresence::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.rich_presence {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CMsgClientCheckFileSignature` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientCheckFileSignature {
    /// Field 1.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientCheckFileSignature {
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

/// `CMsgClientCheckFileSignatureResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientCheckFileSignatureResponse {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub pid: Option<u32>,
    /// Field 3.
    pub eresult: Option<u32>,
    /// Field 4.
    pub filename: Option<String>,
    /// Field 5.
    pub esignatureresult: Option<u32>,
    /// Field 6.
    pub sha_file: Option<Vec<u8>>,
    /// Field 7.
    pub signatureheader: Option<Vec<u8>>,
    /// Field 8.
    pub filesize: Option<u32>,
    /// Field 9.
    pub getlasterror: Option<u32>,
    /// Field 10.
    pub evalvesignaturecheckdetail: Option<u32>,
}

impl Message for CMsgClientCheckFileSignatureResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.pid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.filename = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.esignatureresult = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.sha_file = Some(decoder.read_bytes()?.to_vec());
                }
                7 => {
                    self.signatureheader = Some(decoder.read_bytes()?.to_vec());
                }
                8 => {
                    self.filesize = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.getlasterror = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.evalvesignaturecheckdetail = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.pid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.eresult {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.filename {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.esignatureresult {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.sha_file {
            encoder.write_bytes_field(6, value);
        }
        if let Some(value) = &self.signatureheader {
            encoder.write_bytes_field(7, value);
        }
        if let Some(value) = &self.filesize {
            encoder.write_varint_field(8, u64::from(*value));
        }
        if let Some(value) = &self.getlasterror {
            encoder.write_varint_field(9, u64::from(*value));
        }
        if let Some(value) = &self.evalvesignaturecheckdetail {
            encoder.write_varint_field(10, u64::from(*value));
        }
    }
}

/// `CMsgClientRegisterKey` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRegisterKey {
    /// Field 1.
    pub key: Option<String>,
}

impl Message for CMsgClientRegisterKey {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.key = Some(decoder.read_string()?.to_owned());
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
    }
}

/// `CMsgClientPurchaseResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPurchaseResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub purchase_result_details: Option<i32>,
    /// Field 3.
    pub purchase_receipt_info: Option<Vec<u8>>,
}

impl CMsgClientPurchaseResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientPurchaseResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.purchase_result_details = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.purchase_receipt_info = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.purchase_result_details {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.purchase_receipt_info {
            encoder.write_bytes_field(3, value);
        }
    }
}

/// `CMsgClientActivateOEMLicense` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientActivateOEMLicense {
    /// Field 1.
    pub bios_manufacturer: Option<String>,
    /// Field 2.
    pub bios_serialnumber: Option<String>,
    /// Field 3.
    pub license_file: Option<Vec<u8>>,
    /// Field 4.
    pub mainboard_manufacturer: Option<String>,
    /// Field 5.
    pub mainboard_product: Option<String>,
    /// Field 6.
    pub mainboard_serialnumber: Option<String>,
}

impl Message for CMsgClientActivateOEMLicense {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.bios_manufacturer = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.bios_serialnumber = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.license_file = Some(decoder.read_bytes()?.to_vec());
                }
                4 => {
                    self.mainboard_manufacturer = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.mainboard_product = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.mainboard_serialnumber = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.bios_manufacturer {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.bios_serialnumber {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.license_file {
            encoder.write_bytes_field(3, value);
        }
        if let Some(value) = &self.mainboard_manufacturer {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.mainboard_product {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.mainboard_serialnumber {
            encoder.write_string_field(6, value);
        }
    }
}

/// `CMsgClientRegisterOEMMachine` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRegisterOEMMachine {
    /// Field 1.
    pub oem_register_file: Option<Vec<u8>>,
}

impl Message for CMsgClientRegisterOEMMachine {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.oem_register_file = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.oem_register_file {
            encoder.write_bytes_field(1, value);
        }
    }
}

/// `CMsgClientRegisterOEMMachineResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRegisterOEMMachineResponse {
    /// Field 1.
    pub eresult: Option<u32>,
}

impl Message for CMsgClientRegisterOEMMachineResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
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
    }
}

/// `CMsgClientPurchaseWithMachineID` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPurchaseWithMachineID {
    /// Field 1.
    pub package_id: Option<u32>,
    /// Field 2.
    pub machine_info: Option<Vec<u8>>,
}

impl Message for CMsgClientPurchaseWithMachineID {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.package_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.machine_info = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.machine_info {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgTrading_InitiateTradeRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgTrading_InitiateTradeRequest {
    /// Field 1.
    pub trade_request_id: Option<u32>,
    /// Field 2.
    pub other_steamid: Option<u64>,
    /// Field 3.
    pub other_name: Option<String>,
}

impl Message for CMsgTrading_InitiateTradeRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.trade_request_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.other_steamid = Some(decoder.read_varint()?);
                }
                3 => {
                    self.other_name = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.trade_request_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.other_steamid {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.other_name {
            encoder.write_string_field(3, value);
        }
    }
}

/// `CMsgTrading_InitiateTradeResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgTrading_InitiateTradeResponse {
    /// Field 1.
    pub response: Option<u32>,
    /// Field 2.
    pub trade_request_id: Option<u32>,
    /// Field 3.
    pub other_steamid: Option<u64>,
    /// Field 4.
    pub steamguard_required_days: Option<u32>,
    /// Field 5.
    pub new_device_cooldown_days: Option<u32>,
    /// Field 6.
    pub default_password_reset_probation_days: Option<u32>,
    /// Field 7.
    pub password_reset_probation_days: Option<u32>,
    /// Field 8.
    pub default_email_change_probation_days: Option<u32>,
    /// Field 9.
    pub email_change_probation_days: Option<u32>,
}

impl Message for CMsgTrading_InitiateTradeResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.response = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.trade_request_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.other_steamid = Some(decoder.read_varint()?);
                }
                4 => {
                    self.steamguard_required_days = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.new_device_cooldown_days = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.default_password_reset_probation_days =
                        Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.password_reset_probation_days = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.default_email_change_probation_days = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.email_change_probation_days = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.response {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.trade_request_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.other_steamid {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.steamguard_required_days {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.new_device_cooldown_days {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.default_password_reset_probation_days {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.password_reset_probation_days {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.default_email_change_probation_days {
            encoder.write_varint_field(8, u64::from(*value));
        }
        if let Some(value) = &self.email_change_probation_days {
            encoder.write_varint_field(9, u64::from(*value));
        }
    }
}

/// `CMsgTrading_CancelTradeRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgTrading_CancelTradeRequest {
    /// Field 1.
    pub other_steamid: Option<u64>,
}

impl Message for CMsgTrading_CancelTradeRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.other_steamid = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.other_steamid {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CMsgTrading_StartSession` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgTrading_StartSession {
    /// Field 1.
    pub other_steamid: Option<u64>,
}

impl Message for CMsgTrading_StartSession {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.other_steamid = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.other_steamid {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CMsgClientGetDepotDecryptionKey` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetDepotDecryptionKey {
    /// Field 1.
    pub depot_id: Option<u32>,
    /// Field 2.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientGetDepotDecryptionKey {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientGetDepotDecryptionKeyResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetDepotDecryptionKeyResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub depot_id: Option<u32>,
    /// Field 3.
    pub depot_encryption_key: Option<Vec<u8>>,
}

impl CMsgClientGetDepotDecryptionKeyResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientGetDepotDecryptionKeyResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.depot_encryption_key = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.depot_encryption_key {
            encoder.write_bytes_field(3, value);
        }
    }
}

/// `CMsgClientCheckAppBetaPassword` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientCheckAppBetaPassword {
    /// Field 1.
    pub app_id: Option<u32>,
    /// Field 2.
    pub betapassword: Option<String>,
    /// Field 3.
    pub language: Option<i32>,
}

impl Message for CMsgClientCheckAppBetaPassword {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.betapassword = Some(decoder.read_string()?.to_owned());
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
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.betapassword {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(3, *value);
        }
    }
}

/// Types nested inside [`CMsgClientCheckAppBetaPasswordResponse`].
pub mod c_msg_client_check_app_beta_password_response {
    use super::*;

    /// `BetaPassword` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct BetaPassword {
        /// Field 1.
        pub betaname: Option<String>,
        /// Field 2.
        pub betapassword: Option<String>,
        /// Field 3.
        pub betadescription: Option<String>,
    }

    impl Message for BetaPassword {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.betaname = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.betapassword = Some(decoder.read_string()?.to_owned());
                    }
                    3 => {
                        self.betadescription = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.betaname {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.betapassword {
                encoder.write_string_field(2, value);
            }
            if let Some(value) = &self.betadescription {
                encoder.write_string_field(3, value);
            }
        }
    }
}

/// `CMsgClientCheckAppBetaPasswordResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientCheckAppBetaPasswordResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 4.
    pub betapasswords: Vec<crate::steammessages_clientserver_2::c_msg_client_check_app_beta_password_response::BetaPassword>,
}

impl CMsgClientCheckAppBetaPasswordResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientCheckAppBetaPasswordResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.betapasswords.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_check_app_beta_password_response::BetaPassword::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        for value in &self.betapasswords {
            encoder.write_message_field(4, value);
        }
    }
}

/// `CMsgClientUGSGetGlobalStats` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientUGSGetGlobalStats {
    /// Field 1.
    pub gameid: Option<u64>,
    /// Field 2.
    pub history_days_requested: Option<u32>,
    /// Field 3.
    pub time_last_requested: Option<u32>,
    /// Field 4.
    pub first_day_cached: Option<u32>,
    /// Field 5.
    pub days_cached: Option<u32>,
}

impl Message for CMsgClientUGSGetGlobalStats {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.gameid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.history_days_requested = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.time_last_requested = Some(decoder.read_fixed32()?);
                }
                4 => {
                    self.first_day_cached = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.days_cached = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.gameid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.history_days_requested {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.time_last_requested {
            encoder.write_fixed32_field(3, *value);
        }
        if let Some(value) = &self.first_day_cached {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.days_cached {
            encoder.write_varint_field(5, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientUGSGetGlobalStatsResponse`].
pub mod c_msg_client_ugs_get_global_stats_response {
    use super::*;

    /// Types nested inside [`Day`].
    pub mod day {
        use super::*;

        /// `Stat` — generated from Valve's schema.
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct Stat {
            /// Field 1.
            pub stat_id: Option<i32>,
            /// Field 2.
            pub data: Option<i64>,
        }

        impl Message for Stat {
            fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
                while let Some(key) = decoder.read_key()? {
                    match key.number {
                        1 => {
                            self.stat_id = Some(decoder.read_varint()? as i32);
                        }
                        2 => {
                            self.data = Some(decoder.read_varint()? as i64);
                        }
                        _ => decoder.skip_field(key.wire_type)?,
                    }
                }
                Ok(())
            }

            fn encode_raw(&self, encoder: &mut Encoder) {
                if let Some(value) = &self.stat_id {
                    encoder.write_int32_field(1, *value);
                }
                if let Some(value) = &self.data {
                    encoder.write_varint_field(2, *value as u64);
                }
            }
        }
    }

    /// `Day` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Day {
        /// Field 1.
        pub day_id: Option<u32>,
        /// Field 2.
        pub stats: Vec<crate::steammessages_clientserver_2::c_msg_client_ugs_get_global_stats_response::day::Stat>,
    }

    impl Message for Day {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.day_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.stats.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_ugs_get_global_stats_response::day::Stat::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.day_id {
                encoder.write_varint_field(1, u64::from(*value));
            }
            for value in &self.stats {
                encoder.write_message_field(2, value);
            }
        }
    }
}

/// `CMsgClientUGSGetGlobalStatsResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientUGSGetGlobalStatsResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub timestamp: Option<u32>,
    /// Field 3.
    pub day_current: Option<i32>,
    /// Field 4.
    pub days:
        Vec<crate::steammessages_clientserver_2::c_msg_client_ugs_get_global_stats_response::Day>,
}

impl CMsgClientUGSGetGlobalStatsResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientUGSGetGlobalStatsResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.timestamp = Some(decoder.read_fixed32()?);
                }
                3 => {
                    self.day_current = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.days.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_ugs_get_global_stats_response::Day::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        if let Some(value) = &self.timestamp {
            encoder.write_fixed32_field(2, *value);
        }
        if let Some(value) = &self.day_current {
            encoder.write_int32_field(3, *value);
        }
        for value in &self.days {
            encoder.write_message_field(4, value);
        }
    }
}

/// `CMsgClientRedeemGuestPass` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRedeemGuestPass {
    /// Field 1.
    pub guest_pass_id: Option<u64>,
}

impl Message for CMsgClientRedeemGuestPass {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.guest_pass_id = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.guest_pass_id {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// `CMsgClientRedeemGuestPassResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRedeemGuestPassResponse {
    /// Field 1.
    pub eresult: Option<u32>,
    /// Field 2.
    pub package_id: Option<u32>,
    /// Field 3.
    pub must_own_appid: Option<u32>,
}

impl CMsgClientRedeemGuestPassResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> u32 {
        self.eresult.unwrap_or(2_u32)
    }
}

impl Message for CMsgClientRedeemGuestPassResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.package_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.must_own_appid = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.package_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.must_own_appid {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CMsgClientGetClanActivityCounts` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetClanActivityCounts {
    /// Field 1.
    pub steamid_clans: Vec<u64>,
}

impl Message for CMsgClientGetClanActivityCounts {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.steamid_clans,
                    |d: &mut Decoder<'_>| d.read_varint(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.steamid_clans {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CMsgClientGetClanActivityCountsResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetClanActivityCountsResponse {
    /// Field 1.
    pub eresult: Option<u32>,
}

impl CMsgClientGetClanActivityCountsResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> u32 {
        self.eresult.unwrap_or(2_u32)
    }
}

impl Message for CMsgClientGetClanActivityCountsResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
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
    }
}

/// `CMsgClientOGSReportString` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientOGSReportString {
    /// Field 1.
    pub accumulated: Option<bool>,
    /// Field 2.
    pub sessionid: Option<u64>,
    /// Field 3.
    pub severity: Option<i32>,
    /// Field 4.
    pub formatter: Option<String>,
    /// Field 5.
    pub varargs: Option<Vec<u8>>,
}

impl Message for CMsgClientOGSReportString {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.accumulated = Some(decoder.read_bool()?);
                }
                2 => {
                    self.sessionid = Some(decoder.read_varint()?);
                }
                3 => {
                    self.severity = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.formatter = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.varargs = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.accumulated {
            encoder.write_bool_field(1, *value);
        }
        if let Some(value) = &self.sessionid {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.severity {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.formatter {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.varargs {
            encoder.write_bytes_field(5, value);
        }
    }
}

/// `CMsgClientOGSReportBug` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientOGSReportBug {
    /// Field 1.
    pub sessionid: Option<u64>,
    /// Field 2.
    pub bugtext: Option<String>,
    /// Field 3.
    pub screenshot: Option<Vec<u8>>,
}

impl Message for CMsgClientOGSReportBug {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.sessionid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.bugtext = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.screenshot = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.sessionid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.bugtext {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.screenshot {
            encoder.write_bytes_field(3, value);
        }
    }
}

/// `CMsgClientSentLogs` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientSentLogs {}

impl Message for CMsgClientSentLogs {
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

/// Types nested inside [`CMsgGCClient`].
pub mod c_msg_gc_client {
    use super::*;

    /// `EFlag`, as a newtype so an unrecognised value round-trips instead of
    /// being rejected. Valve adds values without warning.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[repr(transparent)]
    pub struct EFlag(pub i32);

    impl EFlag {
        /// `VALVE_DS` = `1`
        pub const VALVE_DS: Self = Self(1);
        /// The underlying value, as it appears on the wire.
        #[must_use]
        pub const fn value(self) -> i32 {
            self.0
        }
    }

    impl From<i32> for EFlag {
        fn from(value: i32) -> Self {
            Self(value)
        }
    }
}

/// `CMsgGCClient` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgGCClient {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub msgtype: Option<u32>,
    /// Field 3.
    pub payload: Option<Vec<u8>>,
    /// Field 4.
    pub steamid: Option<u64>,
    /// Field 5.
    pub gcname: Option<String>,
    /// Field 6.
    pub ip: Option<u32>,
    /// Field 7.
    pub flags: Option<u32>,
}

impl Message for CMsgGCClient {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.msgtype = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.payload = Some(decoder.read_bytes()?.to_vec());
                }
                4 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                5 => {
                    self.gcname = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.ip = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.flags = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.msgtype {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.payload {
            encoder.write_bytes_field(3, value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(4, *value);
        }
        if let Some(value) = &self.gcname {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.ip {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.flags {
            encoder.write_varint_field(7, u64::from(*value));
        }
    }
}

/// `CMsgClientRequestFreeLicense` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestFreeLicense {
    /// Field 2.
    pub appids: Vec<u32>,
}

impl Message for CMsgClientRequestFreeLicense {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
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
        for value in &self.appids {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientRequestFreeLicenseResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestFreeLicenseResponse {
    /// Field 1.
    pub eresult: Option<u32>,
    /// Field 2.
    pub granted_packageids: Vec<u32>,
    /// Field 3.
    pub granted_appids: Vec<u32>,
}

impl CMsgClientRequestFreeLicenseResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> u32 {
        self.eresult.unwrap_or(2_u32)
    }
}

impl Message for CMsgClientRequestFreeLicenseResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.granted_packageids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.granted_appids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.eresult {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.granted_packageids {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.granted_appids {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CMsgDRMDownloadRequestWithCrashData` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgDRMDownloadRequestWithCrashData {
    /// Field 1.
    pub download_flags: Option<u32>,
    /// Field 2.
    pub download_types_known: Option<u32>,
    /// Field 3.
    pub guid_drm: Option<Vec<u8>>,
    /// Field 4.
    pub guid_split: Option<Vec<u8>>,
    /// Field 5.
    pub guid_merge: Option<Vec<u8>>,
    /// Field 6.
    pub module_name: Option<String>,
    /// Field 7.
    pub module_path: Option<String>,
    /// Field 8.
    pub crash_data: Option<Vec<u8>>,
}

impl Message for CMsgDRMDownloadRequestWithCrashData {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.download_flags = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.download_types_known = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.guid_drm = Some(decoder.read_bytes()?.to_vec());
                }
                4 => {
                    self.guid_split = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.guid_merge = Some(decoder.read_bytes()?.to_vec());
                }
                6 => {
                    self.module_name = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.module_path = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.crash_data = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.download_flags {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.download_types_known {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.guid_drm {
            encoder.write_bytes_field(3, value);
        }
        if let Some(value) = &self.guid_split {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.guid_merge {
            encoder.write_bytes_field(5, value);
        }
        if let Some(value) = &self.module_name {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.module_path {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.crash_data {
            encoder.write_bytes_field(8, value);
        }
    }
}

/// `CMsgDRMDownloadResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgDRMDownloadResponse {
    /// Field 1.
    pub eresult: Option<u32>,
    /// Field 2.
    pub app_id: Option<u32>,
    /// Field 3.
    pub blob_download_type: Option<u32>,
    /// Field 4.
    pub merge_guid: Option<Vec<u8>>,
    /// Field 5.
    pub download_file_dfs_ip: Option<u32>,
    /// Field 6.
    pub download_file_dfs_port: Option<u32>,
    /// Field 7.
    pub download_file_url: Option<String>,
    /// Field 8.
    pub module_path: Option<String>,
}

impl CMsgDRMDownloadResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> u32 {
        self.eresult.unwrap_or(2_u32)
    }
}

impl Message for CMsgDRMDownloadResponse {
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
                    self.blob_download_type = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.merge_guid = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.download_file_dfs_ip = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.download_file_dfs_port = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.download_file_url = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.module_path = Some(decoder.read_string()?.to_owned());
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
        if let Some(value) = &self.blob_download_type {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.merge_guid {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.download_file_dfs_ip {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.download_file_dfs_port {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.download_file_url {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.module_path {
            encoder.write_string_field(8, value);
        }
    }
}

/// `CMsgDRMFinalResult` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgDRMFinalResult {
    /// Field 1.
    pub e_result: Option<u32>,
    /// Field 2.
    pub app_id: Option<u32>,
    /// Field 3.
    pub blob_download_type: Option<u32>,
    /// Field 4.
    pub error_detail: Option<u32>,
    /// Field 5.
    pub merge_guid: Option<Vec<u8>>,
    /// Field 6.
    pub download_file_dfs_ip: Option<u32>,
    /// Field 7.
    pub download_file_dfs_port: Option<u32>,
    /// Field 8.
    pub download_file_url: Option<String>,
}

impl CMsgDRMFinalResult {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn e_result_or_default(&self) -> u32 {
        self.e_result.unwrap_or(2_u32)
    }
}

impl Message for CMsgDRMFinalResult {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.e_result = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.blob_download_type = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.error_detail = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.merge_guid = Some(decoder.read_bytes()?.to_vec());
                }
                6 => {
                    self.download_file_dfs_ip = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.download_file_dfs_port = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.download_file_url = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.e_result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.blob_download_type {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.error_detail {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.merge_guid {
            encoder.write_bytes_field(5, value);
        }
        if let Some(value) = &self.download_file_dfs_ip {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.download_file_dfs_port {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.download_file_url {
            encoder.write_string_field(8, value);
        }
    }
}

/// `CMsgClientDPCheckSpecialSurvey` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDPCheckSpecialSurvey {
    /// Field 1.
    pub survey_id: Option<u32>,
}

impl Message for CMsgClientDPCheckSpecialSurvey {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.survey_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.survey_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
    }
}

/// `CMsgClientDPCheckSpecialSurveyResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDPCheckSpecialSurveyResponse {
    /// Field 1.
    pub e_result: Option<u32>,
    /// Field 2.
    pub state: Option<u32>,
    /// Field 3.
    pub name: Option<String>,
    /// Field 4.
    pub custom_url: Option<String>,
    /// Field 5.
    pub include_software: Option<bool>,
    /// Field 6.
    pub token: Option<Vec<u8>>,
}

impl CMsgClientDPCheckSpecialSurveyResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn e_result_or_default(&self) -> u32 {
        self.e_result.unwrap_or(2_u32)
    }
}

impl Message for CMsgClientDPCheckSpecialSurveyResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.e_result = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.state = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.name = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.custom_url = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.include_software = Some(decoder.read_bool()?);
                }
                6 => {
                    self.token = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.e_result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.state {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.name {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.custom_url {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.include_software {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.token {
            encoder.write_bytes_field(6, value);
        }
    }
}

/// `CMsgClientDPSendSpecialSurveyResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDPSendSpecialSurveyResponse {
    /// Field 1.
    pub survey_id: Option<u32>,
    /// Field 2.
    pub data: Option<Vec<u8>>,
}

impl Message for CMsgClientDPSendSpecialSurveyResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.survey_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.data = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.survey_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.data {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgClientDPSendSpecialSurveyResponseReply` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDPSendSpecialSurveyResponseReply {
    /// Field 1.
    pub e_result: Option<u32>,
    /// Field 2.
    pub token: Option<Vec<u8>>,
}

impl CMsgClientDPSendSpecialSurveyResponseReply {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn e_result_or_default(&self) -> u32 {
        self.e_result.unwrap_or(2_u32)
    }
}

impl Message for CMsgClientDPSendSpecialSurveyResponseReply {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.e_result = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.token = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.e_result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.token {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgClientRequestForgottenPasswordEmail` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestForgottenPasswordEmail {
    /// Field 1.
    pub account_name: Option<String>,
    /// Field 2.
    pub password_tried: Option<String>,
}

impl Message for CMsgClientRequestForgottenPasswordEmail {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.account_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.password_tried = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.account_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.password_tried {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CMsgClientRequestForgottenPasswordEmailResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestForgottenPasswordEmailResponse {
    /// Field 1.
    pub e_result: Option<u32>,
    /// Field 2.
    pub use_secret_question: Option<bool>,
}

impl Message for CMsgClientRequestForgottenPasswordEmailResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.e_result = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.use_secret_question = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.e_result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.use_secret_question {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// Types nested inside [`CMsgClientItemAnnouncements`].
pub mod c_msg_client_item_announcements {
    use super::*;

    /// `UnseenItem` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct UnseenItem {
        /// Field 1.
        pub appid: Option<u32>,
        /// Field 2.
        pub context_id: Option<u64>,
        /// Field 3.
        pub asset_id: Option<u64>,
        /// Field 4.
        pub amount: Option<u64>,
        /// Field 5.
        pub rtime32_gained: Option<u32>,
        /// Field 6.
        pub source_appid: Option<u32>,
    }

    impl Message for UnseenItem {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.appid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.context_id = Some(decoder.read_varint()?);
                    }
                    3 => {
                        self.asset_id = Some(decoder.read_varint()?);
                    }
                    4 => {
                        self.amount = Some(decoder.read_varint()?);
                    }
                    5 => {
                        self.rtime32_gained = Some(decoder.read_fixed32()?);
                    }
                    6 => {
                        self.source_appid = Some(decoder.read_varint()? as u32);
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
            if let Some(value) = &self.context_id {
                encoder.write_varint_field(2, *value);
            }
            if let Some(value) = &self.asset_id {
                encoder.write_varint_field(3, *value);
            }
            if let Some(value) = &self.amount {
                encoder.write_varint_field(4, *value);
            }
            if let Some(value) = &self.rtime32_gained {
                encoder.write_fixed32_field(5, *value);
            }
            if let Some(value) = &self.source_appid {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientItemAnnouncements` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientItemAnnouncements {
    /// Field 1.
    pub count_new_items: Option<u32>,
    /// Field 2.
    pub unseen_items:
        Vec<crate::steammessages_clientserver_2::c_msg_client_item_announcements::UnseenItem>,
}

impl Message for CMsgClientItemAnnouncements {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.count_new_items = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.unseen_items.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_item_announcements::UnseenItem::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.count_new_items {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.unseen_items {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgClientRequestItemAnnouncements` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestItemAnnouncements {}

impl Message for CMsgClientRequestItemAnnouncements {
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

/// Types nested inside [`CMsgClientUserNotifications`].
pub mod c_msg_client_user_notifications {
    use super::*;

    /// `Notification` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Notification {
        /// Field 1.
        pub user_notification_type: Option<u32>,
        /// Field 2.
        pub count: Option<u32>,
    }

    impl Message for Notification {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.user_notification_type = Some(decoder.read_varint()? as u32);
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
            if let Some(value) = &self.user_notification_type {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.count {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientUserNotifications` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientUserNotifications {
    /// Field 1.
    pub notifications:
        Vec<crate::steammessages_clientserver_2::c_msg_client_user_notifications::Notification>,
}

impl Message for CMsgClientUserNotifications {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.notifications.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_user_notifications::Notification::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.notifications {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CMsgClientCommentNotifications` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientCommentNotifications {
    /// Field 1.
    pub count_new_comments: Option<u32>,
    /// Field 2.
    pub count_new_comments_owner: Option<u32>,
    /// Field 3.
    pub count_new_comments_subscriptions: Option<u32>,
}

impl Message for CMsgClientCommentNotifications {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.count_new_comments = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.count_new_comments_owner = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.count_new_comments_subscriptions = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.count_new_comments {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.count_new_comments_owner {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.count_new_comments_subscriptions {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CMsgClientRequestCommentNotifications` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestCommentNotifications {}

impl Message for CMsgClientRequestCommentNotifications {
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

/// `CMsgClientOfflineMessageNotification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientOfflineMessageNotification {
    /// Field 1.
    pub offline_messages: Option<u32>,
    /// Field 2.
    pub friends_with_offline_messages: Vec<u32>,
}

impl Message for CMsgClientOfflineMessageNotification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.offline_messages = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.friends_with_offline_messages,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.offline_messages {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.friends_with_offline_messages {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientRequestOfflineMessageCount` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientRequestOfflineMessageCount {}

impl Message for CMsgClientRequestOfflineMessageCount {
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

/// `CMsgClientChatGetFriendMessageHistory` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientChatGetFriendMessageHistory {
    /// Field 1.
    pub steamid: Option<u64>,
}

impl Message for CMsgClientChatGetFriendMessageHistory {
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

/// Types nested inside [`CMsgClientChatGetFriendMessageHistoryResponse`].
pub mod c_msg_client_chat_get_friend_message_history_response {
    use super::*;

    /// `FriendMessage` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct FriendMessage {
        /// Field 1.
        pub accountid: Option<u32>,
        /// Field 2.
        pub timestamp: Option<u32>,
        /// Field 3.
        pub message: Option<String>,
        /// Field 4.
        pub unread: Option<bool>,
    }

    impl Message for FriendMessage {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.accountid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.timestamp = Some(decoder.read_varint()? as u32);
                    }
                    3 => {
                        self.message = Some(decoder.read_string()?.to_owned());
                    }
                    4 => {
                        self.unread = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.accountid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.timestamp {
                encoder.write_varint_field(2, u64::from(*value));
            }
            if let Some(value) = &self.message {
                encoder.write_string_field(3, value);
            }
            if let Some(value) = &self.unread {
                encoder.write_bool_field(4, *value);
            }
        }
    }
}

/// `CMsgClientChatGetFriendMessageHistoryResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientChatGetFriendMessageHistoryResponse {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub success: Option<u32>,
    /// Field 3.
    pub messages: Vec<crate::steammessages_clientserver_2::c_msg_client_chat_get_friend_message_history_response::FriendMessage>,
}

impl Message for CMsgClientChatGetFriendMessageHistoryResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.success = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.messages.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_chat_get_friend_message_history_response::FriendMessage::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        if let Some(value) = &self.success {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.messages {
            encoder.write_message_field(3, value);
        }
    }
}

/// `CMsgClientChatGetFriendMessageHistoryForOfflineMessages` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientChatGetFriendMessageHistoryForOfflineMessages {}

impl Message for CMsgClientChatGetFriendMessageHistoryForOfflineMessages {
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

/// `CMsgClientFSGetFriendsSteamLevels` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientFSGetFriendsSteamLevels {
    /// Field 1.
    pub accountids: Vec<u32>,
}

impl Message for CMsgClientFSGetFriendsSteamLevels {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.accountids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.accountids {
            encoder.write_varint_field(1, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientFSGetFriendsSteamLevelsResponse`].
pub mod c_msg_client_fs_get_friends_steam_levels_response {
    use super::*;

    /// `Friend` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Friend {
        /// Field 1.
        pub accountid: Option<u32>,
        /// Field 2.
        pub level: Option<u32>,
    }

    impl Message for Friend {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.accountid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.level = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.accountid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.level {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientFSGetFriendsSteamLevelsResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientFSGetFriendsSteamLevelsResponse {
    /// Field 1.
    pub friends: Vec<crate::steammessages_clientserver_2::c_msg_client_fs_get_friends_steam_levels_response::Friend>,
}

impl Message for CMsgClientFSGetFriendsSteamLevelsResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.friends.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_fs_get_friends_steam_levels_response::Friend::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.friends {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CMsgClientEmailAddrInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientEmailAddrInfo {
    /// Field 1.
    pub email_address: Option<String>,
    /// Field 2.
    pub email_is_validated: Option<bool>,
    /// Field 3.
    pub email_validation_changed: Option<bool>,
    /// Field 4.
    pub credential_change_requires_code: Option<bool>,
    /// Field 5.
    pub password_or_secretqa_change_requires_code: Option<bool>,
}

impl Message for CMsgClientEmailAddrInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.email_address = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.email_is_validated = Some(decoder.read_bool()?);
                }
                3 => {
                    self.email_validation_changed = Some(decoder.read_bool()?);
                }
                4 => {
                    self.credential_change_requires_code = Some(decoder.read_bool()?);
                }
                5 => {
                    self.password_or_secretqa_change_requires_code = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.email_address {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.email_is_validated {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.email_validation_changed {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.credential_change_requires_code {
            encoder.write_bool_field(4, *value);
        }
        if let Some(value) = &self.password_or_secretqa_change_requires_code {
            encoder.write_bool_field(5, *value);
        }
    }
}

/// Types nested inside [`CMsgCREItemVoteSummary`].
pub mod c_msg_cre_item_vote_summary {
    use super::*;

    /// `PublishedFileId` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PublishedFileId {
        /// Field 1.
        pub published_file_id: Option<u64>,
    }

    impl Message for PublishedFileId {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
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
        }
    }
}

/// `CMsgCREItemVoteSummary` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgCREItemVoteSummary {
    /// Field 1.
    pub published_file_ids:
        Vec<crate::steammessages_clientserver_2::c_msg_cre_item_vote_summary::PublishedFileId>,
}

impl Message for CMsgCREItemVoteSummary {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.published_file_ids.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_cre_item_vote_summary::PublishedFileId::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.published_file_ids {
            encoder.write_message_field(1, value);
        }
    }
}

/// Types nested inside [`CMsgCREItemVoteSummaryResponse`].
pub mod c_msg_cre_item_vote_summary_response {
    use super::*;

    /// `ItemVoteSummary` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ItemVoteSummary {
        /// Field 1.
        pub published_file_id: Option<u64>,
        /// Field 2.
        pub votes_for: Option<i32>,
        /// Field 3.
        pub votes_against: Option<i32>,
        /// Field 4.
        pub reports: Option<i32>,
        /// Field 5.
        pub score: Option<f32>,
    }

    impl Message for ItemVoteSummary {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.votes_for = Some(decoder.read_varint()? as i32);
                    }
                    3 => {
                        self.votes_against = Some(decoder.read_varint()? as i32);
                    }
                    4 => {
                        self.reports = Some(decoder.read_varint()? as i32);
                    }
                    5 => {
                        self.score = Some(decoder.read_float()?);
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
            if let Some(value) = &self.votes_for {
                encoder.write_int32_field(2, *value);
            }
            if let Some(value) = &self.votes_against {
                encoder.write_int32_field(3, *value);
            }
            if let Some(value) = &self.reports {
                encoder.write_int32_field(4, *value);
            }
            if let Some(value) = &self.score {
                encoder.write_float_field(5, *value);
            }
        }
    }
}

/// `CMsgCREItemVoteSummaryResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgCREItemVoteSummaryResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub item_vote_summaries: Vec<
        crate::steammessages_clientserver_2::c_msg_cre_item_vote_summary_response::ItemVoteSummary,
    >,
}

impl CMsgCREItemVoteSummaryResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgCREItemVoteSummaryResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.item_vote_summaries.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_cre_item_vote_summary_response::ItemVoteSummary::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        for value in &self.item_vote_summaries {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgCREUpdateUserPublishedItemVote` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgCREUpdateUserPublishedItemVote {
    /// Field 1.
    pub published_file_id: Option<u64>,
    /// Field 2.
    pub vote_up: Option<bool>,
}

impl Message for CMsgCREUpdateUserPublishedItemVote {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.published_file_id = Some(decoder.read_fixed64()?);
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
        if let Some(value) = &self.published_file_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.vote_up {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// `CMsgCREUpdateUserPublishedItemVoteResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgCREUpdateUserPublishedItemVoteResponse {
    /// Field 1.
    pub eresult: Option<i32>,
}

impl CMsgCREUpdateUserPublishedItemVoteResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgCREUpdateUserPublishedItemVoteResponse {
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

/// Types nested inside [`CMsgCREGetUserPublishedItemVoteDetails`].
pub mod c_msg_cre_get_user_published_item_vote_details {
    use super::*;

    /// `PublishedFileId` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct PublishedFileId {
        /// Field 1.
        pub published_file_id: Option<u64>,
    }

    impl Message for PublishedFileId {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
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
        }
    }
}

/// `CMsgCREGetUserPublishedItemVoteDetails` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgCREGetUserPublishedItemVoteDetails {
    /// Field 1.
    pub published_file_ids: Vec<crate::steammessages_clientserver_2::c_msg_cre_get_user_published_item_vote_details::PublishedFileId>,
}

impl Message for CMsgCREGetUserPublishedItemVoteDetails {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.published_file_ids.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_cre_get_user_published_item_vote_details::PublishedFileId::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.published_file_ids {
            encoder.write_message_field(1, value);
        }
    }
}

/// Types nested inside [`CMsgCREGetUserPublishedItemVoteDetailsResponse`].
pub mod c_msg_cre_get_user_published_item_vote_details_response {
    use super::*;

    /// `UserItemVoteDetail` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct UserItemVoteDetail {
        /// Field 1.
        pub published_file_id: Option<u64>,
        /// Field 2.
        pub vote: Option<i32>,
    }

    impl UserItemVoteDetail {
        /// Field 2 , or its schema default when absent.
        #[must_use]
        pub fn vote_or_default(&self) -> i32 {
            self.vote.unwrap_or(0_i32)
        }
    }

    impl Message for UserItemVoteDetail {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.published_file_id = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.vote = Some(decoder.read_varint()? as i32);
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
            if let Some(value) = &self.vote {
                encoder.write_int32_field(2, *value);
            }
        }
    }
}

/// `CMsgCREGetUserPublishedItemVoteDetailsResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgCREGetUserPublishedItemVoteDetailsResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub user_item_vote_details: Vec<crate::steammessages_clientserver_2::c_msg_cre_get_user_published_item_vote_details_response::UserItemVoteDetail>,
}

impl CMsgCREGetUserPublishedItemVoteDetailsResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgCREGetUserPublishedItemVoteDetailsResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.user_item_vote_details.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_cre_get_user_published_item_vote_details_response::UserItemVoteDetail::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        for value in &self.user_item_vote_details {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgFSGetFollowerCount` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgFSGetFollowerCount {
    /// Field 1.
    pub steam_id: Option<u64>,
}

impl Message for CMsgFSGetFollowerCount {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id = Some(decoder.read_fixed64()?);
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
    }
}

/// `CMsgFSGetFollowerCountResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgFSGetFollowerCountResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub count: Option<i32>,
}

impl CMsgFSGetFollowerCountResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn count_or_default(&self) -> i32 {
        self.count.unwrap_or(0_i32)
    }
}

impl Message for CMsgFSGetFollowerCountResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.count = Some(decoder.read_varint()? as i32);
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
        if let Some(value) = &self.count {
            encoder.write_int32_field(2, *value);
        }
    }
}

/// `CMsgFSGetIsFollowing` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgFSGetIsFollowing {
    /// Field 1.
    pub steam_id: Option<u64>,
}

impl Message for CMsgFSGetIsFollowing {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steam_id = Some(decoder.read_fixed64()?);
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
    }
}

/// `CMsgFSGetIsFollowingResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgFSGetIsFollowingResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub is_following: Option<bool>,
}

impl CMsgFSGetIsFollowingResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn is_following_or_default(&self) -> bool {
        self.is_following.unwrap_or(false)
    }
}

impl Message for CMsgFSGetIsFollowingResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.is_following = Some(decoder.read_bool()?);
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
        if let Some(value) = &self.is_following {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// `CMsgFSEnumerateFollowingList` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgFSEnumerateFollowingList {
    /// Field 1.
    pub start_index: Option<u32>,
}

impl Message for CMsgFSEnumerateFollowingList {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.start_index = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.start_index {
            encoder.write_varint_field(1, u64::from(*value));
        }
    }
}

/// `CMsgFSEnumerateFollowingListResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgFSEnumerateFollowingListResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub total_results: Option<i32>,
    /// Field 3.
    pub steam_ids: Vec<u64>,
}

impl CMsgFSEnumerateFollowingListResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgFSEnumerateFollowingListResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.total_results = Some(decoder.read_varint()? as i32);
                }
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.steam_ids,
                    |d: &mut Decoder<'_>| d.read_fixed64(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.eresult {
            encoder.write_int32_field(1, *value);
        }
        if let Some(value) = &self.total_results {
            encoder.write_int32_field(2, *value);
        }
        for value in &self.steam_ids {
            encoder.write_fixed64_field(3, *value);
        }
    }
}

/// `CMsgDPGetNumberOfCurrentPlayers` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgDPGetNumberOfCurrentPlayers {
    /// Field 1.
    pub appid: Option<u32>,
}

impl Message for CMsgDPGetNumberOfCurrentPlayers {
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

/// `CMsgDPGetNumberOfCurrentPlayersResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgDPGetNumberOfCurrentPlayersResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub player_count: Option<i32>,
}

impl CMsgDPGetNumberOfCurrentPlayersResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgDPGetNumberOfCurrentPlayersResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.player_count = Some(decoder.read_varint()? as i32);
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
        if let Some(value) = &self.player_count {
            encoder.write_int32_field(2, *value);
        }
    }
}

/// `CMsgClientFriendUserStatusPublished` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientFriendUserStatusPublished {
    /// Field 1.
    pub friend_steamid: Option<u64>,
    /// Field 2.
    pub appid: Option<u32>,
    /// Field 3.
    pub status_text: Option<String>,
}

impl Message for CMsgClientFriendUserStatusPublished {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.friend_steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.status_text = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.friend_steamid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.status_text {
            encoder.write_string_field(3, value);
        }
    }
}

/// `CMsgClientServiceMethodLegacy` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServiceMethodLegacy {
    /// Field 1.
    pub method_name: Option<String>,
    /// Field 2.
    pub serialized_method: Option<Vec<u8>>,
    /// Field 3.
    pub is_notification: Option<bool>,
}

impl Message for CMsgClientServiceMethodLegacy {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.method_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.serialized_method = Some(decoder.read_bytes()?.to_vec());
                }
                3 => {
                    self.is_notification = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.method_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.serialized_method {
            encoder.write_bytes_field(2, value);
        }
        if let Some(value) = &self.is_notification {
            encoder.write_bool_field(3, *value);
        }
    }
}

/// `CMsgClientServiceMethodLegacyResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServiceMethodLegacyResponse {
    /// Field 1.
    pub method_name: Option<String>,
    /// Field 2.
    pub serialized_method_response: Option<Vec<u8>>,
}

impl Message for CMsgClientServiceMethodLegacyResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.method_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.serialized_method_response = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.method_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.serialized_method_response {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgClientUIMode` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientUIMode {
    /// Field 1.
    pub uimode: Option<u32>,
    /// Field 2.
    pub chat_mode: Option<u32>,
}

impl Message for CMsgClientUIMode {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.uimode = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.chat_mode = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.uimode {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.chat_mode {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientVanityURLChangedNotification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientVanityURLChangedNotification {
    /// Field 1.
    pub vanity_url: Option<String>,
}

impl Message for CMsgClientVanityURLChangedNotification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.vanity_url = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.vanity_url {
            encoder.write_string_field(1, value);
        }
    }
}

/// `CMsgClientAuthorizeLocalDeviceRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAuthorizeLocalDeviceRequest {
    /// Field 1.
    pub device_description: Option<String>,
    /// Field 2.
    pub owner_account_id: Option<u32>,
    /// Field 3.
    pub local_device_token: Option<u64>,
}

impl Message for CMsgClientAuthorizeLocalDeviceRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.device_description = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.owner_account_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.local_device_token = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.device_description {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.owner_account_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.local_device_token {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CMsgClientAuthorizeLocalDevice` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAuthorizeLocalDevice {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub owner_account_id: Option<u32>,
    /// Field 3.
    pub authed_device_token: Option<u64>,
}

impl CMsgClientAuthorizeLocalDevice {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientAuthorizeLocalDevice {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.owner_account_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.authed_device_token = Some(decoder.read_varint()?);
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
        if let Some(value) = &self.owner_account_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.authed_device_token {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CMsgClientAuthorizeLocalDeviceNotification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientAuthorizeLocalDeviceNotification {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub owner_account_id: Option<u32>,
    /// Field 3.
    pub local_device_token: Option<u64>,
}

impl CMsgClientAuthorizeLocalDeviceNotification {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientAuthorizeLocalDeviceNotification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.owner_account_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.local_device_token = Some(decoder.read_varint()?);
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
        if let Some(value) = &self.owner_account_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.local_device_token {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CMsgClientDeauthorizeDeviceRequest` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDeauthorizeDeviceRequest {
    /// Field 1.
    pub deauthorization_account_id: Option<u32>,
    /// Field 2.
    pub deauthorization_device_token: Option<u64>,
}

impl Message for CMsgClientDeauthorizeDeviceRequest {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.deauthorization_account_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.deauthorization_device_token = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.deauthorization_account_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.deauthorization_device_token {
            encoder.write_varint_field(2, *value);
        }
    }
}

/// `CMsgClientDeauthorizeDevice` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientDeauthorizeDevice {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub deauthorization_account_id: Option<u32>,
}

impl CMsgClientDeauthorizeDevice {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientDeauthorizeDevice {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.deauthorization_account_id = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.deauthorization_account_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientUseLocalDeviceAuthorizations`].
pub mod c_msg_client_use_local_device_authorizations {
    use super::*;

    /// `DeviceToken` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct DeviceToken {
        /// Field 1.
        pub owner_account_id: Option<u32>,
        /// Field 2.
        pub token_id: Option<u64>,
    }

    impl Message for DeviceToken {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.owner_account_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.token_id = Some(decoder.read_varint()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.owner_account_id {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.token_id {
                encoder.write_varint_field(2, *value);
            }
        }
    }
}

/// `CMsgClientUseLocalDeviceAuthorizations` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientUseLocalDeviceAuthorizations {
    /// Field 1.
    pub authorization_account_id: Vec<u32>,
    /// Field 2.
    pub device_tokens: Vec<crate::steammessages_clientserver_2::c_msg_client_use_local_device_authorizations::DeviceToken>,
}

impl Message for CMsgClientUseLocalDeviceAuthorizations {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.authorization_account_id,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                2 => {
                    self.device_tokens.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_use_local_device_authorizations::DeviceToken::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.authorization_account_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.device_tokens {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgClientGetAuthorizedDevices` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetAuthorizedDevices {}

impl Message for CMsgClientGetAuthorizedDevices {
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

/// Types nested inside [`CMsgClientGetAuthorizedDevicesResponse`].
pub mod c_msg_client_get_authorized_devices_response {
    use super::*;

    /// `AuthorizedDevice` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct AuthorizedDevice {
        /// Field 1.
        pub auth_device_token: Option<u64>,
        /// Field 2.
        pub device_name: Option<String>,
        /// Field 3.
        pub last_access_time: Option<u32>,
        /// Field 4.
        pub borrower_id: Option<u32>,
        /// Field 5.
        pub is_pending: Option<bool>,
        /// Field 6.
        pub app_played: Option<u32>,
    }

    impl Message for AuthorizedDevice {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.auth_device_token = Some(decoder.read_varint()?);
                    }
                    2 => {
                        self.device_name = Some(decoder.read_string()?.to_owned());
                    }
                    3 => {
                        self.last_access_time = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.borrower_id = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.is_pending = Some(decoder.read_bool()?);
                    }
                    6 => {
                        self.app_played = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.auth_device_token {
                encoder.write_varint_field(1, *value);
            }
            if let Some(value) = &self.device_name {
                encoder.write_string_field(2, value);
            }
            if let Some(value) = &self.last_access_time {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.borrower_id {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.is_pending {
                encoder.write_bool_field(5, *value);
            }
            if let Some(value) = &self.app_played {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientGetAuthorizedDevicesResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetAuthorizedDevicesResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub authorized_device: Vec<crate::steammessages_clientserver_2::c_msg_client_get_authorized_devices_response::AuthorizedDevice>,
}

impl CMsgClientGetAuthorizedDevicesResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientGetAuthorizedDevicesResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.authorized_device.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_get_authorized_devices_response::AuthorizedDevice::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
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
        for value in &self.authorized_device {
            encoder.write_message_field(2, value);
        }
    }
}

/// Types nested inside [`CMsgClientSharedLibraryLockStatus`].
pub mod c_msg_client_shared_library_lock_status {
    use super::*;

    /// `LockedLibrary` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct LockedLibrary {
        /// Field 1.
        pub owner_id: Option<u32>,
        /// Field 2.
        pub locked_by: Option<u32>,
    }

    impl Message for LockedLibrary {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.owner_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.locked_by = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.owner_id {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.locked_by {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientSharedLibraryLockStatus` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientSharedLibraryLockStatus {
    /// Field 1.
    pub locked_library: Vec<
        crate::steammessages_clientserver_2::c_msg_client_shared_library_lock_status::LockedLibrary,
    >,
    /// Field 2.
    pub own_library_locked_by: Option<u32>,
}

impl Message for CMsgClientSharedLibraryLockStatus {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.locked_library.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_shared_library_lock_status::LockedLibrary::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.own_library_locked_by = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.locked_library {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.own_library_locked_by {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgClientSharedLibraryStopPlaying`].
pub mod c_msg_client_shared_library_stop_playing {
    use super::*;

    /// `StopApp` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct StopApp {
        /// Field 1.
        pub app_id: Option<u32>,
        /// Field 2.
        pub owner_id: Option<u32>,
    }

    impl Message for StopApp {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.app_id = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.owner_id = Some(decoder.read_varint()? as u32);
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
            if let Some(value) = &self.owner_id {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `CMsgClientSharedLibraryStopPlaying` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientSharedLibraryStopPlaying {
    /// Field 1.
    pub seconds_left: Option<i32>,
    /// Field 2.
    pub stop_apps:
        Vec<crate::steammessages_clientserver_2::c_msg_client_shared_library_stop_playing::StopApp>,
}

impl Message for CMsgClientSharedLibraryStopPlaying {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.seconds_left = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.stop_apps.push({ let mut nested = crate::steammessages_clientserver_2::c_msg_client_shared_library_stop_playing::StopApp::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.seconds_left {
            encoder.write_int32_field(1, *value);
        }
        for value in &self.stop_apps {
            encoder.write_message_field(2, value);
        }
    }
}

/// `CMsgClientServiceCall` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServiceCall {
    /// Field 1.
    pub sysid_routing: Option<Vec<u8>>,
    /// Field 2.
    pub call_handle: Option<u32>,
    /// Field 3.
    pub module_crc: Option<u32>,
    /// Field 4.
    pub module_hash: Option<Vec<u8>>,
    /// Field 5.
    pub function_id: Option<u32>,
    /// Field 6.
    pub cub_output_max: Option<u32>,
    /// Field 7.
    pub flags: Option<u32>,
    /// Field 8.
    pub callparameter: Option<Vec<u8>>,
    /// Field 9.
    pub ping_only: Option<bool>,
    /// Field 10.
    pub max_outstanding_calls: Option<u32>,
    /// Field 11.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientServiceCall {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.sysid_routing = Some(decoder.read_bytes()?.to_vec());
                }
                2 => {
                    self.call_handle = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.module_crc = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.module_hash = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.function_id = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.cub_output_max = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.flags = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.callparameter = Some(decoder.read_bytes()?.to_vec());
                }
                9 => {
                    self.ping_only = Some(decoder.read_bool()?);
                }
                10 => {
                    self.max_outstanding_calls = Some(decoder.read_varint()? as u32);
                }
                11 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.sysid_routing {
            encoder.write_bytes_field(1, value);
        }
        if let Some(value) = &self.call_handle {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.module_crc {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.module_hash {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.function_id {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.cub_output_max {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.flags {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.callparameter {
            encoder.write_bytes_field(8, value);
        }
        if let Some(value) = &self.ping_only {
            encoder.write_bool_field(9, *value);
        }
        if let Some(value) = &self.max_outstanding_calls {
            encoder.write_varint_field(10, u64::from(*value));
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(11, u64::from(*value));
        }
    }
}

/// `CMsgClientServiceModule` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServiceModule {
    /// Field 1.
    pub module_crc: Option<u32>,
    /// Field 2.
    pub module_hash: Option<Vec<u8>>,
    /// Field 3.
    pub module_content: Option<Vec<u8>>,
}

impl Message for CMsgClientServiceModule {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.module_crc = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.module_hash = Some(decoder.read_bytes()?.to_vec());
                }
                3 => {
                    self.module_content = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.module_crc {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.module_hash {
            encoder.write_bytes_field(2, value);
        }
        if let Some(value) = &self.module_content {
            encoder.write_bytes_field(3, value);
        }
    }
}

/// `CMsgClientServiceCallResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientServiceCallResponse {
    /// Field 1.
    pub sysid_routing: Option<Vec<u8>>,
    /// Field 2.
    pub call_handle: Option<u32>,
    /// Field 3.
    pub module_crc: Option<u32>,
    /// Field 4.
    pub module_hash: Option<Vec<u8>>,
    /// Field 5.
    pub ecallresult: Option<u32>,
    /// Field 6.
    pub result_content: Option<Vec<u8>>,
    /// Field 7.
    pub os_version_info: Option<Vec<u8>>,
    /// Field 8.
    pub system_info: Option<Vec<u8>>,
    /// Field 9.
    pub load_address: Option<u64>,
    /// Field 10.
    pub exception_record: Option<Vec<u8>>,
    /// Field 11.
    pub portable_os_version_info: Option<Vec<u8>>,
    /// Field 12.
    pub portable_system_info: Option<Vec<u8>>,
    /// Field 13.
    pub was_converted: Option<bool>,
    /// Field 14.
    pub internal_result: Option<u32>,
    /// Field 15.
    pub current_count: Option<u32>,
    /// Field 16.
    pub last_call_handle: Option<u32>,
    /// Field 17.
    pub last_call_module_crc: Option<u32>,
    /// Field 18.
    pub last_call_sysid_routing: Option<Vec<u8>>,
    /// Field 19.
    pub last_ecallresult: Option<u32>,
    /// Field 20.
    pub last_callissue_delta: Option<u32>,
    /// Field 21.
    pub last_callcomplete_delta: Option<u32>,
}

impl Message for CMsgClientServiceCallResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.sysid_routing = Some(decoder.read_bytes()?.to_vec());
                }
                2 => {
                    self.call_handle = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.module_crc = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.module_hash = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.ecallresult = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.result_content = Some(decoder.read_bytes()?.to_vec());
                }
                7 => {
                    self.os_version_info = Some(decoder.read_bytes()?.to_vec());
                }
                8 => {
                    self.system_info = Some(decoder.read_bytes()?.to_vec());
                }
                9 => {
                    self.load_address = Some(decoder.read_fixed64()?);
                }
                10 => {
                    self.exception_record = Some(decoder.read_bytes()?.to_vec());
                }
                11 => {
                    self.portable_os_version_info = Some(decoder.read_bytes()?.to_vec());
                }
                12 => {
                    self.portable_system_info = Some(decoder.read_bytes()?.to_vec());
                }
                13 => {
                    self.was_converted = Some(decoder.read_bool()?);
                }
                14 => {
                    self.internal_result = Some(decoder.read_varint()? as u32);
                }
                15 => {
                    self.current_count = Some(decoder.read_varint()? as u32);
                }
                16 => {
                    self.last_call_handle = Some(decoder.read_varint()? as u32);
                }
                17 => {
                    self.last_call_module_crc = Some(decoder.read_varint()? as u32);
                }
                18 => {
                    self.last_call_sysid_routing = Some(decoder.read_bytes()?.to_vec());
                }
                19 => {
                    self.last_ecallresult = Some(decoder.read_varint()? as u32);
                }
                20 => {
                    self.last_callissue_delta = Some(decoder.read_varint()? as u32);
                }
                21 => {
                    self.last_callcomplete_delta = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.sysid_routing {
            encoder.write_bytes_field(1, value);
        }
        if let Some(value) = &self.call_handle {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.module_crc {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.module_hash {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.ecallresult {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.result_content {
            encoder.write_bytes_field(6, value);
        }
        if let Some(value) = &self.os_version_info {
            encoder.write_bytes_field(7, value);
        }
        if let Some(value) = &self.system_info {
            encoder.write_bytes_field(8, value);
        }
        if let Some(value) = &self.load_address {
            encoder.write_fixed64_field(9, *value);
        }
        if let Some(value) = &self.exception_record {
            encoder.write_bytes_field(10, value);
        }
        if let Some(value) = &self.portable_os_version_info {
            encoder.write_bytes_field(11, value);
        }
        if let Some(value) = &self.portable_system_info {
            encoder.write_bytes_field(12, value);
        }
        if let Some(value) = &self.was_converted {
            encoder.write_bool_field(13, *value);
        }
        if let Some(value) = &self.internal_result {
            encoder.write_varint_field(14, u64::from(*value));
        }
        if let Some(value) = &self.current_count {
            encoder.write_varint_field(15, u64::from(*value));
        }
        if let Some(value) = &self.last_call_handle {
            encoder.write_varint_field(16, u64::from(*value));
        }
        if let Some(value) = &self.last_call_module_crc {
            encoder.write_varint_field(17, u64::from(*value));
        }
        if let Some(value) = &self.last_call_sysid_routing {
            encoder.write_bytes_field(18, value);
        }
        if let Some(value) = &self.last_ecallresult {
            encoder.write_varint_field(19, u64::from(*value));
        }
        if let Some(value) = &self.last_callissue_delta {
            encoder.write_varint_field(20, u64::from(*value));
        }
        if let Some(value) = &self.last_callcomplete_delta {
            encoder.write_varint_field(21, u64::from(*value));
        }
    }
}

/// `CMsgAMUnlockH264` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgAMUnlockH264 {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub platform: Option<i32>,
    /// Field 3.
    pub reason: Option<i32>,
}

impl Message for CMsgAMUnlockH264 {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.platform = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.reason = Some(decoder.read_varint()? as i32);
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
        if let Some(value) = &self.platform {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.reason {
            encoder.write_int32_field(3, *value);
        }
    }
}

/// `CMsgAMUnlockH264Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgAMUnlockH264Response {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub encryption_key: Option<Vec<u8>>,
}

impl CMsgAMUnlockH264Response {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgAMUnlockH264Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.encryption_key = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.encryption_key {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgClientPlayingSessionState` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPlayingSessionState {
    /// Field 2.
    pub playing_blocked: Option<bool>,
    /// Field 3.
    pub playing_app: Option<u32>,
}

impl Message for CMsgClientPlayingSessionState {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                2 => {
                    self.playing_blocked = Some(decoder.read_bool()?);
                }
                3 => {
                    self.playing_app = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.playing_blocked {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.playing_app {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}

/// `CMsgClientKickPlayingSession` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientKickPlayingSession {
    /// Field 1.
    pub only_stop_game: Option<bool>,
}

impl Message for CMsgClientKickPlayingSession {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.only_stop_game = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.only_stop_game {
            encoder.write_bool_field(1, *value);
        }
    }
}

/// `CMsgClientVoiceCallPreAuthorize` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientVoiceCallPreAuthorize {
    /// Field 1.
    pub caller_steamid: Option<u64>,
    /// Field 2.
    pub receiver_steamid: Option<u64>,
    /// Field 3.
    pub caller_id: Option<i32>,
    /// Field 4.
    pub hangup: Option<bool>,
}

impl Message for CMsgClientVoiceCallPreAuthorize {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.caller_steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.receiver_steamid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.caller_id = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.hangup = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.caller_steamid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.receiver_steamid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.caller_id {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.hangup {
            encoder.write_bool_field(4, *value);
        }
    }
}

/// `CMsgClientVoiceCallPreAuthorizeResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientVoiceCallPreAuthorizeResponse {
    /// Field 1.
    pub caller_steamid: Option<u64>,
    /// Field 2.
    pub receiver_steamid: Option<u64>,
    /// Field 3.
    pub eresult: Option<i32>,
    /// Field 4.
    pub caller_id: Option<i32>,
}

impl CMsgClientVoiceCallPreAuthorizeResponse {
    /// Field 3 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientVoiceCallPreAuthorizeResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.caller_steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.receiver_steamid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.caller_id = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.caller_steamid {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.receiver_steamid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.eresult {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.caller_id {
            encoder.write_int32_field(4, *value);
        }
    }
}

/// `CMsgBadgeCraftedNotification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgBadgeCraftedNotification {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub badge_level: Option<u32>,
}

impl Message for CMsgBadgeCraftedNotification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.badge_level = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.badge_level {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientStartPeerContentServer` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientStartPeerContentServer {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub client_remote_id: Option<u64>,
    /// Field 3.
    pub app_id: Option<u32>,
    /// Field 4.
    pub current_build_id: Option<u32>,
}

impl Message for CMsgClientStartPeerContentServer {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.client_remote_id = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.current_build_id = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.client_remote_id {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.current_build_id {
            encoder.write_varint_field(4, u64::from(*value));
        }
    }
}

/// `CMsgClientStartPeerContentServerResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientStartPeerContentServerResponse {
    /// Field 1.
    pub result: Option<u32>,
    /// Field 2.
    pub server_port: Option<u32>,
    /// Field 3.
    pub installed_depots: Vec<u32>,
    /// Field 4.
    pub access_token: Option<u64>,
}

impl Message for CMsgClientStartPeerContentServerResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.result = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.server_port = Some(decoder.read_varint()? as u32);
                }
                3 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.installed_depots,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                4 => {
                    self.access_token = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.server_port {
            encoder.write_varint_field(2, u64::from(*value));
        }
        for value in &self.installed_depots {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.access_token {
            encoder.write_varint_field(4, *value);
        }
    }
}

/// `CMsgClientGetPeerContentInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetPeerContentInfo {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub client_remote_id: Option<u64>,
    /// Field 3.
    pub owned_games_visible: Option<bool>,
}

impl Message for CMsgClientGetPeerContentInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.client_remote_id = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.owned_games_visible = Some(decoder.read_bool()?);
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
        if let Some(value) = &self.client_remote_id {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.owned_games_visible {
            encoder.write_bool_field(3, *value);
        }
    }
}

/// `CMsgClientGetPeerContentInfoResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientGetPeerContentInfoResponse {
    /// Field 1.
    pub result: Option<u32>,
    /// Field 2.
    pub apps: Vec<u32>,
}

impl Message for CMsgClientGetPeerContentInfoResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.result = Some(decoder.read_varint()? as u32);
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.apps,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.result {
            encoder.write_varint_field(1, u64::from(*value));
        }
        for value in &self.apps {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CMsgClientPendingGameLaunch` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPendingGameLaunch {
    /// Field 1.
    pub app_id: Option<u32>,
}

impl Message for CMsgClientPendingGameLaunch {
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

/// `CMsgClientPendingGameLaunchResponse` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgClientPendingGameLaunchResponse {
    /// Field 1.
    pub eresult: Option<i32>,
    /// Field 2.
    pub app_id: Option<u32>,
    /// Field 3.
    pub envkey: Option<String>,
}

impl CMsgClientPendingGameLaunchResponse {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
}

impl Message for CMsgClientPendingGameLaunchResponse {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.app_id = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.envkey = Some(decoder.read_string()?.to_owned());
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
        if let Some(value) = &self.app_id {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.envkey {
            encoder.write_string_field(3, value);
        }
    }
}
