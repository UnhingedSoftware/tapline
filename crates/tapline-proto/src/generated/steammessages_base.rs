//! Generated from `steammessages_base.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EBanContentCheckResult`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBanContentCheckResult(pub i32);

impl EBanContentCheckResult {
    /// `k_EBanContentCheckResult_NotScanned` = `0`
    pub const k_EBanContentCheckResult_NotScanned: Self = Self(0);
    /// `k_EBanContentCheckResult_Reset` = `1`
    pub const k_EBanContentCheckResult_Reset: Self = Self(1);
    /// `k_EBanContentCheckResult_NeedsChecking` = `2`
    pub const k_EBanContentCheckResult_NeedsChecking: Self = Self(2);
    /// `k_EBanContentCheckResult_VeryUnlikely` = `5`
    pub const k_EBanContentCheckResult_VeryUnlikely: Self = Self(5);
    /// `k_EBanContentCheckResult_Unlikely` = `30`
    pub const k_EBanContentCheckResult_Unlikely: Self = Self(30);
    /// `k_EBanContentCheckResult_Possible` = `50`
    pub const k_EBanContentCheckResult_Possible: Self = Self(50);
    /// `k_EBanContentCheckResult_Likely` = `75`
    pub const k_EBanContentCheckResult_Likely: Self = Self(75);
    /// `k_EBanContentCheckResult_VeryLikely` = `100`
    pub const k_EBanContentCheckResult_VeryLikely: Self = Self(100);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBanContentCheckResult {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EProtoClanEventType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EProtoClanEventType(pub i32);

impl EProtoClanEventType {
    /// `k_EClanOtherEvent` = `1`
    pub const k_EClanOtherEvent: Self = Self(1);
    /// `k_EClanGameEvent` = `2`
    pub const k_EClanGameEvent: Self = Self(2);
    /// `k_EClanPartyEvent` = `3`
    pub const k_EClanPartyEvent: Self = Self(3);
    /// `k_EClanMeetingEvent` = `4`
    pub const k_EClanMeetingEvent: Self = Self(4);
    /// `k_EClanSpecialCauseEvent` = `5`
    pub const k_EClanSpecialCauseEvent: Self = Self(5);
    /// `k_EClanMusicAndArtsEvent` = `6`
    pub const k_EClanMusicAndArtsEvent: Self = Self(6);
    /// `k_EClanSportsEvent` = `7`
    pub const k_EClanSportsEvent: Self = Self(7);
    /// `k_EClanTripEvent` = `8`
    pub const k_EClanTripEvent: Self = Self(8);
    /// `k_EClanChatEvent` = `9`
    pub const k_EClanChatEvent: Self = Self(9);
    /// `k_EClanGameReleaseEvent` = `10`
    pub const k_EClanGameReleaseEvent: Self = Self(10);
    /// `k_EClanBroadcastEvent` = `11`
    pub const k_EClanBroadcastEvent: Self = Self(11);
    /// `k_EClanSmallUpdateEvent` = `12`
    pub const k_EClanSmallUpdateEvent: Self = Self(12);
    /// `k_EClanPreAnnounceMajorUpdateEvent` = `13`
    pub const k_EClanPreAnnounceMajorUpdateEvent: Self = Self(13);
    /// `k_EClanMajorUpdateEvent` = `14`
    pub const k_EClanMajorUpdateEvent: Self = Self(14);
    /// `k_EClanDLCReleaseEvent` = `15`
    pub const k_EClanDLCReleaseEvent: Self = Self(15);
    /// `k_EClanFutureReleaseEvent` = `16`
    pub const k_EClanFutureReleaseEvent: Self = Self(16);
    /// `k_EClanESportTournamentStreamEvent` = `17`
    pub const k_EClanESportTournamentStreamEvent: Self = Self(17);
    /// `k_EClanDevStreamEvent` = `18`
    pub const k_EClanDevStreamEvent: Self = Self(18);
    /// `k_EClanFamousStreamEvent` = `19`
    pub const k_EClanFamousStreamEvent: Self = Self(19);
    /// `k_EClanGameSalesEvent` = `20`
    pub const k_EClanGameSalesEvent: Self = Self(20);
    /// `k_EClanGameItemSalesEvent` = `21`
    pub const k_EClanGameItemSalesEvent: Self = Self(21);
    /// `k_EClanInGameBonusXPEvent` = `22`
    pub const k_EClanInGameBonusXPEvent: Self = Self(22);
    /// `k_EClanInGameLootEvent` = `23`
    pub const k_EClanInGameLootEvent: Self = Self(23);
    /// `k_EClanInGamePerksEvent` = `24`
    pub const k_EClanInGamePerksEvent: Self = Self(24);
    /// `k_EClanInGameChallengeEvent` = `25`
    pub const k_EClanInGameChallengeEvent: Self = Self(25);
    /// `k_EClanInGameContestEvent` = `26`
    pub const k_EClanInGameContestEvent: Self = Self(26);
    /// `k_EClanIRLEvent` = `27`
    pub const k_EClanIRLEvent: Self = Self(27);
    /// `k_EClanNewsEvent` = `28`
    pub const k_EClanNewsEvent: Self = Self(28);
    /// `k_EClanBetaReleaseEvent` = `29`
    pub const k_EClanBetaReleaseEvent: Self = Self(29);
    /// `k_EClanInGameContentReleaseEvent` = `30`
    pub const k_EClanInGameContentReleaseEvent: Self = Self(30);
    /// `k_EClanFreeTrial` = `31`
    pub const k_EClanFreeTrial: Self = Self(31);
    /// `k_EClanSeasonRelease` = `32`
    pub const k_EClanSeasonRelease: Self = Self(32);
    /// `k_EClanSeasonUpdate` = `33`
    pub const k_EClanSeasonUpdate: Self = Self(33);
    /// `k_EClanCrosspostEvent` = `34`
    pub const k_EClanCrosspostEvent: Self = Self(34);
    /// `k_EClanInGameEventGeneral` = `35`
    pub const k_EClanInGameEventGeneral: Self = Self(35);
    /// `k_EClanCreatorHome` = `36`
    pub const k_EClanCreatorHome: Self = Self(36);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EProtoClanEventType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `PartnerEventNotificationType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct PartnerEventNotificationType(pub i32);

impl PartnerEventNotificationType {
    /// `k_EEventStart` = `0`
    pub const k_EEventStart: Self = Self(0);
    /// `k_EEventBroadcastStart` = `1`
    pub const k_EEventBroadcastStart: Self = Self(1);
    /// `k_EEventMatchStart` = `2`
    pub const k_EEventMatchStart: Self = Self(2);
    /// `k_EEventPartnerMaxType` = `3`
    pub const k_EEventPartnerMaxType: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for PartnerEventNotificationType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `CMsgIPAddress` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgIPAddress {
    /// Field 1.
    pub v4: Option<u32>,
    /// Field 2.
    pub v6: Option<Vec<u8>>,
}

impl Message for CMsgIPAddress {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.v4 = Some(decoder.read_fixed32()?);
                }
                2 => {
                    self.v6 = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.v4 {
            encoder.write_fixed32_field(1, *value);
        }
        if let Some(value) = &self.v6 {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgIPAddressBucket` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgIPAddressBucket {
    /// Field 1.
    pub original_ip_address: Option<crate::steammessages_base::CMsgIPAddress>,
    /// Field 2.
    pub bucket: Option<u64>,
}

impl Message for CMsgIPAddressBucket {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.original_ip_address = Some({
                        let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                2 => {
                    self.bucket = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.original_ip_address {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.bucket {
            encoder.write_fixed64_field(2, *value);
        }
    }
}

/// `CMsgGCRoutingProtoBufHeader` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgGCRoutingProtoBufHeader {
    /// Field 1.
    pub dst_gcid_queue: Option<u64>,
    /// Field 2.
    pub dst_gc_dir_index: Option<u32>,
}

impl Message for CMsgGCRoutingProtoBufHeader {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.dst_gcid_queue = Some(decoder.read_varint()?);
                }
                2 => {
                    self.dst_gc_dir_index = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.dst_gcid_queue {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.dst_gc_dir_index {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// Types nested inside [`CMsgProtoBufHeader`].
pub mod c_msg_proto_buf_header {
    use super::*;

    /// `ESessionDisposition`, as a newtype so an unrecognised value round-trips instead of
    /// being rejected. Valve adds values without warning.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[repr(transparent)]
    pub struct ESessionDisposition(pub i32);

    impl ESessionDisposition {
        /// `k_ESessionDispositionNormal` = `0`
        pub const k_ESessionDispositionNormal: Self = Self(0);
        /// `k_ESessionDispositionDisconnect` = `1`
        pub const k_ESessionDispositionDisconnect: Self = Self(1);
        /// The underlying value, as it appears on the wire.
        #[must_use]
        pub const fn value(self) -> i32 {
            self.0
        }
    }

    impl From<i32> for ESessionDisposition {
        fn from(value: i32) -> Self {
            Self(value)
        }
    }
}

/// `CMsgProtoBufHeader` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgProtoBufHeader {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub client_sessionid: Option<i32>,
    /// Field 3.
    pub routing_appid: Option<u32>,
    /// Field 10.
    pub jobid_source: Option<u64>,
    /// Field 11.
    pub jobid_target: Option<u64>,
    /// Field 12.
    pub target_job_name: Option<String>,
    /// Field 24.
    pub seq_num: Option<i32>,
    /// Field 13.
    pub eresult: Option<i32>,
    /// Field 14.
    pub error_message: Option<String>,
    /// Field 16.
    pub auth_account_flags: Option<u32>,
    /// Field 22.
    pub token_source: Option<u32>,
    /// Field 23.
    pub admin_spoofing_user: Option<bool>,
    /// Field 17.
    pub transport_error: Option<i32>,
    /// Field 18.
    pub messageid: Option<u64>,
    /// Field 19.
    pub publisher_group_id: Option<u32>,
    /// Field 20.
    pub sysid: Option<u32>,
    /// Field 25.
    pub webapi_key_id: Option<u32>,
    /// Field 26.
    pub is_from_external_source: Option<bool>,
    /// Field 27.
    pub forward_to_sysid: Vec<u32>,
    /// Field 28.
    pub cm_sysid: Option<u32>,
    /// Field 31.
    pub launcher_type: Option<u32>,
    /// Field 32.
    pub realm: Option<u32>,
    /// Field 33.
    pub timeout_ms: Option<i32>,
    /// Field 34.
    pub debug_source: Option<String>,
    /// Field 35.
    pub debug_source_string_index: Option<u32>,
    /// Field 36.
    pub token_id: Option<u64>,
    /// Field 37.
    pub routing_gc: Option<crate::steammessages_base::CMsgGCRoutingProtoBufHeader>,
    /// Field 38.
    pub session_disposition:
        Option<crate::steammessages_base::c_msg_proto_buf_header::ESessionDisposition>,
    /// Field 39.
    pub wg_token: Option<String>,
    /// Field 40.
    pub webui_auth_key: Option<String>,
    /// Field 41.
    pub exclude_client_sessionids: Vec<i32>,
    /// Field 43.
    pub admin_request_spoofing_steamid: Option<u64>,
    /// Field 44.
    pub is_valveds: Option<bool>,
    /// Field 45.
    pub trace_tag: Option<u64>,
    /// Field 15.
    pub ip: Option<u32>,
    /// Field 29.
    pub ip_v6: Option<Vec<u8>>,
}

impl CMsgProtoBufHeader {
    /// Field 10 , or its schema default when absent.
    #[must_use]
    pub fn jobid_source_or_default(&self) -> u64 {
        self.jobid_source.unwrap_or(18446744073709551615_u64)
    }
    /// Field 11 , or its schema default when absent.
    #[must_use]
    pub fn jobid_target_or_default(&self) -> u64 {
        self.jobid_target.unwrap_or(18446744073709551615_u64)
    }
    /// Field 13 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> i32 {
        self.eresult.unwrap_or(2_i32)
    }
    /// Field 17 , or its schema default when absent.
    #[must_use]
    pub fn transport_error_or_default(&self) -> i32 {
        self.transport_error.unwrap_or(1_i32)
    }
    /// Field 18 , or its schema default when absent.
    #[must_use]
    pub fn messageid_or_default(&self) -> u64 {
        self.messageid.unwrap_or(18446744073709551615_u64)
    }
    /// Field 31 , or its schema default when absent.
    #[must_use]
    pub fn launcher_type_or_default(&self) -> u32 {
        self.launcher_type.unwrap_or(0_u32)
    }
    /// Field 32 , or its schema default when absent.
    #[must_use]
    pub fn realm_or_default(&self) -> u32 {
        self.realm.unwrap_or(0_u32)
    }
    /// Field 33 , or its schema default when absent.
    #[must_use]
    pub fn timeout_ms_or_default(&self) -> i32 {
        self.timeout_ms.unwrap_or(-1_i32)
    }
    /// Field 38 , or its schema default when absent.
    #[must_use]
    pub fn session_disposition_or_default(
        &self,
    ) -> crate::steammessages_base::c_msg_proto_buf_header::ESessionDisposition {
        self.session_disposition.unwrap_or(crate::steammessages_base::c_msg_proto_buf_header::ESessionDisposition::k_ESessionDispositionNormal)
    }
}

impl Message for CMsgProtoBufHeader {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.client_sessionid = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.routing_appid = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.jobid_source = Some(decoder.read_fixed64()?);
                }
                11 => {
                    self.jobid_target = Some(decoder.read_fixed64()?);
                }
                12 => {
                    self.target_job_name = Some(decoder.read_string()?.to_owned());
                }
                24 => {
                    self.seq_num = Some(decoder.read_varint()? as i32);
                }
                13 => {
                    self.eresult = Some(decoder.read_varint()? as i32);
                }
                14 => {
                    self.error_message = Some(decoder.read_string()?.to_owned());
                }
                16 => {
                    self.auth_account_flags = Some(decoder.read_varint()? as u32);
                }
                22 => {
                    self.token_source = Some(decoder.read_varint()? as u32);
                }
                23 => {
                    self.admin_spoofing_user = Some(decoder.read_bool()?);
                }
                17 => {
                    self.transport_error = Some(decoder.read_varint()? as i32);
                }
                18 => {
                    self.messageid = Some(decoder.read_varint()?);
                }
                19 => {
                    self.publisher_group_id = Some(decoder.read_varint()? as u32);
                }
                20 => {
                    self.sysid = Some(decoder.read_varint()? as u32);
                }
                25 => {
                    self.webapi_key_id = Some(decoder.read_varint()? as u32);
                }
                26 => {
                    self.is_from_external_source = Some(decoder.read_bool()?);
                }
                27 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.forward_to_sysid,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                28 => {
                    self.cm_sysid = Some(decoder.read_varint()? as u32);
                }
                31 => {
                    self.launcher_type = Some(decoder.read_varint()? as u32);
                }
                32 => {
                    self.realm = Some(decoder.read_varint()? as u32);
                }
                33 => {
                    self.timeout_ms = Some(decoder.read_varint()? as i32);
                }
                34 => {
                    self.debug_source = Some(decoder.read_string()?.to_owned());
                }
                35 => {
                    self.debug_source_string_index = Some(decoder.read_varint()? as u32);
                }
                36 => {
                    self.token_id = Some(decoder.read_varint()?);
                }
                37 => {
                    self.routing_gc = Some({
                        let mut nested =
                            crate::steammessages_base::CMsgGCRoutingProtoBufHeader::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                38 => {
                    self.session_disposition = Some(crate::steammessages_base::c_msg_proto_buf_header::ESessionDisposition::from(decoder.read_varint()? as i32));
                }
                39 => {
                    self.wg_token = Some(decoder.read_string()?.to_owned());
                }
                40 => {
                    self.webui_auth_key = Some(decoder.read_string()?.to_owned());
                }
                41 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.exclude_client_sessionids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as i32),
                )?,
                43 => {
                    self.admin_request_spoofing_steamid = Some(decoder.read_fixed64()?);
                }
                44 => {
                    self.is_valveds = Some(decoder.read_bool()?);
                }
                45 => {
                    self.trace_tag = Some(decoder.read_fixed64()?);
                }
                15 => {
                    self.ip = Some(decoder.read_varint()? as u32);
                }
                29 => {
                    self.ip_v6 = Some(decoder.read_bytes()?.to_vec());
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
        if let Some(value) = &self.client_sessionid {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.routing_appid {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.jobid_source {
            encoder.write_fixed64_field(10, *value);
        }
        if let Some(value) = &self.jobid_target {
            encoder.write_fixed64_field(11, *value);
        }
        if let Some(value) = &self.target_job_name {
            encoder.write_string_field(12, value);
        }
        if let Some(value) = &self.seq_num {
            encoder.write_int32_field(24, *value);
        }
        if let Some(value) = &self.eresult {
            encoder.write_int32_field(13, *value);
        }
        if let Some(value) = &self.error_message {
            encoder.write_string_field(14, value);
        }
        if let Some(value) = &self.auth_account_flags {
            encoder.write_varint_field(16, u64::from(*value));
        }
        if let Some(value) = &self.token_source {
            encoder.write_varint_field(22, u64::from(*value));
        }
        if let Some(value) = &self.admin_spoofing_user {
            encoder.write_bool_field(23, *value);
        }
        if let Some(value) = &self.transport_error {
            encoder.write_int32_field(17, *value);
        }
        if let Some(value) = &self.messageid {
            encoder.write_varint_field(18, *value);
        }
        if let Some(value) = &self.publisher_group_id {
            encoder.write_varint_field(19, u64::from(*value));
        }
        if let Some(value) = &self.sysid {
            encoder.write_varint_field(20, u64::from(*value));
        }
        if let Some(value) = &self.webapi_key_id {
            encoder.write_varint_field(25, u64::from(*value));
        }
        if let Some(value) = &self.is_from_external_source {
            encoder.write_bool_field(26, *value);
        }
        for value in &self.forward_to_sysid {
            encoder.write_varint_field(27, u64::from(*value));
        }
        if let Some(value) = &self.cm_sysid {
            encoder.write_varint_field(28, u64::from(*value));
        }
        if let Some(value) = &self.launcher_type {
            encoder.write_varint_field(31, u64::from(*value));
        }
        if let Some(value) = &self.realm {
            encoder.write_varint_field(32, u64::from(*value));
        }
        if let Some(value) = &self.timeout_ms {
            encoder.write_int32_field(33, *value);
        }
        if let Some(value) = &self.debug_source {
            encoder.write_string_field(34, value);
        }
        if let Some(value) = &self.debug_source_string_index {
            encoder.write_varint_field(35, u64::from(*value));
        }
        if let Some(value) = &self.token_id {
            encoder.write_varint_field(36, *value);
        }
        if let Some(value) = &self.routing_gc {
            encoder.write_message_field(37, value);
        }
        if let Some(value) = &self.session_disposition {
            encoder.write_varint_field(38, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.wg_token {
            encoder.write_string_field(39, value);
        }
        if let Some(value) = &self.webui_auth_key {
            encoder.write_string_field(40, value);
        }
        for value in &self.exclude_client_sessionids {
            encoder.write_int32_field(41, *value);
        }
        if let Some(value) = &self.admin_request_spoofing_steamid {
            encoder.write_fixed64_field(43, *value);
        }
        if let Some(value) = &self.is_valveds {
            encoder.write_bool_field(44, *value);
        }
        if let Some(value) = &self.trace_tag {
            encoder.write_fixed64_field(45, *value);
        }
        if let Some(value) = &self.ip {
            encoder.write_varint_field(15, u64::from(*value));
        }
        if let Some(value) = &self.ip_v6 {
            encoder.write_bytes_field(29, value);
        }
    }
}

/// Types nested inside [`CMsgKubeRPCPacket`].
pub mod c_msg_kube_rpc_packet {
    use super::*;

    /// `Hdr` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Hdr {
        /// Field 10.
        pub jobid_source: Option<u64>,
        /// Field 11.
        pub jobid_target: Option<u64>,
        /// Field 13.
        pub eresult: Option<i32>,
        /// Field 12.
        pub target_job_name: Option<String>,
        /// Field 14.
        pub error_message: Option<String>,
        /// Field 42.
        pub reply_address: Option<String>,
    }

    impl Hdr {
        /// Field 10 , or its schema default when absent.
        #[must_use]
        pub fn jobid_source_or_default(&self) -> u64 {
            self.jobid_source.unwrap_or(18446744073709551615_u64)
        }
        /// Field 11 , or its schema default when absent.
        #[must_use]
        pub fn jobid_target_or_default(&self) -> u64 {
            self.jobid_target.unwrap_or(18446744073709551615_u64)
        }
        /// Field 13 , or its schema default when absent.
        #[must_use]
        pub fn eresult_or_default(&self) -> i32 {
            self.eresult.unwrap_or(2_i32)
        }
    }

    impl Message for Hdr {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    10 => {
                        self.jobid_source = Some(decoder.read_fixed64()?);
                    }
                    11 => {
                        self.jobid_target = Some(decoder.read_fixed64()?);
                    }
                    13 => {
                        self.eresult = Some(decoder.read_varint()? as i32);
                    }
                    12 => {
                        self.target_job_name = Some(decoder.read_string()?.to_owned());
                    }
                    14 => {
                        self.error_message = Some(decoder.read_string()?.to_owned());
                    }
                    42 => {
                        self.reply_address = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.jobid_source {
                encoder.write_fixed64_field(10, *value);
            }
            if let Some(value) = &self.jobid_target {
                encoder.write_fixed64_field(11, *value);
            }
            if let Some(value) = &self.eresult {
                encoder.write_int32_field(13, *value);
            }
            if let Some(value) = &self.target_job_name {
                encoder.write_string_field(12, value);
            }
            if let Some(value) = &self.error_message {
                encoder.write_string_field(14, value);
            }
            if let Some(value) = &self.reply_address {
                encoder.write_string_field(42, value);
            }
        }
    }
}

/// `CMsgKubeRPCPacket` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgKubeRPCPacket {
    /// Field 1.
    pub hdr: Option<crate::steammessages_base::c_msg_kube_rpc_packet::Hdr>,
    /// Field 2.
    pub payload: Option<Vec<u8>>,
}

impl Message for CMsgKubeRPCPacket {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.hdr = Some({
                        let mut nested =
                            crate::steammessages_base::c_msg_kube_rpc_packet::Hdr::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                2 => {
                    self.payload = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.hdr {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.payload {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgMulti` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgMulti {
    /// Field 1.
    pub size_unzipped: Option<u32>,
    /// Field 2.
    pub message_body: Option<Vec<u8>>,
}

impl Message for CMsgMulti {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.size_unzipped = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.message_body = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.size_unzipped {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.message_body {
            encoder.write_bytes_field(2, value);
        }
    }
}

/// `CMsgProtobufWrapped` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgProtobufWrapped {
    /// Field 1.
    pub message_body: Option<Vec<u8>>,
}

impl Message for CMsgProtobufWrapped {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.message_body = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.message_body {
            encoder.write_bytes_field(1, value);
        }
    }
}

/// `CMsgAuthTicket` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgAuthTicket {
    /// Field 1.
    pub estate: Option<u32>,
    /// Field 2.
    pub eresult: Option<u32>,
    /// Field 3.
    pub steamid: Option<u64>,
    /// Field 4.
    pub gameid: Option<u64>,
    /// Field 5.
    pub h_steam_pipe: Option<u32>,
    /// Field 6.
    pub ticket_crc: Option<u32>,
    /// Field 7.
    pub ticket: Option<Vec<u8>>,
    /// Field 8.
    pub server_secret: Option<Vec<u8>>,
    /// Field 9.
    pub ticket_type: Option<u32>,
}

impl CMsgAuthTicket {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn eresult_or_default(&self) -> u32 {
        self.eresult.unwrap_or(2_u32)
    }
}

impl Message for CMsgAuthTicket {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.estate = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.eresult = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                4 => {
                    self.gameid = Some(decoder.read_fixed64()?);
                }
                5 => {
                    self.h_steam_pipe = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.ticket_crc = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.ticket = Some(decoder.read_bytes()?.to_vec());
                }
                8 => {
                    self.server_secret = Some(decoder.read_bytes()?.to_vec());
                }
                9 => {
                    self.ticket_type = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.estate {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.eresult {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(3, *value);
        }
        if let Some(value) = &self.gameid {
            encoder.write_fixed64_field(4, *value);
        }
        if let Some(value) = &self.h_steam_pipe {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.ticket_crc {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.ticket {
            encoder.write_bytes_field(7, value);
        }
        if let Some(value) = &self.server_secret {
            encoder.write_bytes_field(8, value);
        }
        if let Some(value) = &self.ticket_type {
            encoder.write_varint_field(9, u64::from(*value));
        }
    }
}

/// `CCDDBAppDetailCommon` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCDDBAppDetailCommon {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub name: Option<String>,
    /// Field 3.
    pub icon: Option<String>,
    /// Field 6.
    pub tool: Option<bool>,
    /// Field 7.
    pub demo: Option<bool>,
    /// Field 8.
    pub media: Option<bool>,
    /// Field 9.
    pub community_visible_stats: Option<bool>,
    /// Field 10.
    pub friendly_name: Option<String>,
    /// Field 11.
    pub propagation: Option<String>,
    /// Field 12.
    pub has_adult_content: Option<bool>,
    /// Field 13.
    pub is_visible_in_steam_china: Option<bool>,
    /// Field 14.
    pub app_type: Option<u32>,
    /// Field 15.
    pub has_adult_content_sex: Option<bool>,
    /// Field 16.
    pub has_adult_content_violence: Option<bool>,
    /// Field 17.
    pub content_descriptorids: Vec<u32>,
    /// Field 18.
    pub content_descriptorids_including_dlc: Vec<u32>,
}

impl Message for CCDDBAppDetailCommon {
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
                    self.icon = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.tool = Some(decoder.read_bool()?);
                }
                7 => {
                    self.demo = Some(decoder.read_bool()?);
                }
                8 => {
                    self.media = Some(decoder.read_bool()?);
                }
                9 => {
                    self.community_visible_stats = Some(decoder.read_bool()?);
                }
                10 => {
                    self.friendly_name = Some(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.propagation = Some(decoder.read_string()?.to_owned());
                }
                12 => {
                    self.has_adult_content = Some(decoder.read_bool()?);
                }
                13 => {
                    self.is_visible_in_steam_china = Some(decoder.read_bool()?);
                }
                14 => {
                    self.app_type = Some(decoder.read_varint()? as u32);
                }
                15 => {
                    self.has_adult_content_sex = Some(decoder.read_bool()?);
                }
                16 => {
                    self.has_adult_content_violence = Some(decoder.read_bool()?);
                }
                17 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.content_descriptorids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                18 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.content_descriptorids_including_dlc,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
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
        if let Some(value) = &self.name {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.icon {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.tool {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.demo {
            encoder.write_bool_field(7, *value);
        }
        if let Some(value) = &self.media {
            encoder.write_bool_field(8, *value);
        }
        if let Some(value) = &self.community_visible_stats {
            encoder.write_bool_field(9, *value);
        }
        if let Some(value) = &self.friendly_name {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.propagation {
            encoder.write_string_field(11, value);
        }
        if let Some(value) = &self.has_adult_content {
            encoder.write_bool_field(12, *value);
        }
        if let Some(value) = &self.is_visible_in_steam_china {
            encoder.write_bool_field(13, *value);
        }
        if let Some(value) = &self.app_type {
            encoder.write_varint_field(14, u64::from(*value));
        }
        if let Some(value) = &self.has_adult_content_sex {
            encoder.write_bool_field(15, *value);
        }
        if let Some(value) = &self.has_adult_content_violence {
            encoder.write_bool_field(16, *value);
        }
        for value in &self.content_descriptorids {
            encoder.write_varint_field(17, u64::from(*value));
        }
        for value in &self.content_descriptorids_including_dlc {
            encoder.write_varint_field(18, u64::from(*value));
        }
    }
}

/// `CMsgAppRights` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgAppRights {
    /// Field 1.
    pub edit_info: Option<bool>,
    /// Field 2.
    pub publish: Option<bool>,
    /// Field 3.
    pub view_error_data: Option<bool>,
    /// Field 4.
    pub download: Option<bool>,
    /// Field 5.
    pub upload_cdkeys: Option<bool>,
    /// Field 6.
    pub generate_cdkeys: Option<bool>,
    /// Field 7.
    pub view_financials: Option<bool>,
    /// Field 8.
    pub manage_ceg: Option<bool>,
    /// Field 9.
    pub manage_signing: Option<bool>,
    /// Field 10.
    pub manage_cdkeys: Option<bool>,
    /// Field 11.
    pub edit_marketing: Option<bool>,
    /// Field 12.
    pub economy_support: Option<bool>,
    /// Field 13.
    pub economy_support_supervisor: Option<bool>,
    /// Field 14.
    pub manage_pricing: Option<bool>,
    /// Field 15.
    pub broadcast_live: Option<bool>,
    /// Field 16.
    pub view_marketing_traffic: Option<bool>,
    /// Field 17.
    pub edit_store_display_content: Option<bool>,
}

impl Message for CMsgAppRights {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.edit_info = Some(decoder.read_bool()?);
                }
                2 => {
                    self.publish = Some(decoder.read_bool()?);
                }
                3 => {
                    self.view_error_data = Some(decoder.read_bool()?);
                }
                4 => {
                    self.download = Some(decoder.read_bool()?);
                }
                5 => {
                    self.upload_cdkeys = Some(decoder.read_bool()?);
                }
                6 => {
                    self.generate_cdkeys = Some(decoder.read_bool()?);
                }
                7 => {
                    self.view_financials = Some(decoder.read_bool()?);
                }
                8 => {
                    self.manage_ceg = Some(decoder.read_bool()?);
                }
                9 => {
                    self.manage_signing = Some(decoder.read_bool()?);
                }
                10 => {
                    self.manage_cdkeys = Some(decoder.read_bool()?);
                }
                11 => {
                    self.edit_marketing = Some(decoder.read_bool()?);
                }
                12 => {
                    self.economy_support = Some(decoder.read_bool()?);
                }
                13 => {
                    self.economy_support_supervisor = Some(decoder.read_bool()?);
                }
                14 => {
                    self.manage_pricing = Some(decoder.read_bool()?);
                }
                15 => {
                    self.broadcast_live = Some(decoder.read_bool()?);
                }
                16 => {
                    self.view_marketing_traffic = Some(decoder.read_bool()?);
                }
                17 => {
                    self.edit_store_display_content = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.edit_info {
            encoder.write_bool_field(1, *value);
        }
        if let Some(value) = &self.publish {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.view_error_data {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.download {
            encoder.write_bool_field(4, *value);
        }
        if let Some(value) = &self.upload_cdkeys {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.generate_cdkeys {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.view_financials {
            encoder.write_bool_field(7, *value);
        }
        if let Some(value) = &self.manage_ceg {
            encoder.write_bool_field(8, *value);
        }
        if let Some(value) = &self.manage_signing {
            encoder.write_bool_field(9, *value);
        }
        if let Some(value) = &self.manage_cdkeys {
            encoder.write_bool_field(10, *value);
        }
        if let Some(value) = &self.edit_marketing {
            encoder.write_bool_field(11, *value);
        }
        if let Some(value) = &self.economy_support {
            encoder.write_bool_field(12, *value);
        }
        if let Some(value) = &self.economy_support_supervisor {
            encoder.write_bool_field(13, *value);
        }
        if let Some(value) = &self.manage_pricing {
            encoder.write_bool_field(14, *value);
        }
        if let Some(value) = &self.broadcast_live {
            encoder.write_bool_field(15, *value);
        }
        if let Some(value) = &self.view_marketing_traffic {
            encoder.write_bool_field(16, *value);
        }
        if let Some(value) = &self.edit_store_display_content {
            encoder.write_bool_field(17, *value);
        }
    }
}

/// `CCuratorPreferences` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCuratorPreferences {
    /// Field 2.
    pub platform_windows: Option<bool>,
    /// Field 3.
    pub platform_mac: Option<bool>,
    /// Field 4.
    pub platform_linux: Option<bool>,
    /// Field 5.
    pub vr_content: Option<bool>,
    /// Field 6.
    pub adult_content_violence: Option<bool>,
    /// Field 7.
    pub adult_content_sex: Option<bool>,
    /// Field 8.
    pub timestamp_updated: Option<u32>,
    /// Field 9.
    pub tagids_curated: Vec<u32>,
    /// Field 10.
    pub tagids_filtered: Vec<u32>,
    /// Field 11.
    pub website_title: Option<String>,
    /// Field 12.
    pub website_url: Option<String>,
    /// Field 13.
    pub discussion_url: Option<String>,
    /// Field 14.
    pub show_broadcast: Option<bool>,
}

impl Message for CCuratorPreferences {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                2 => {
                    self.platform_windows = Some(decoder.read_bool()?);
                }
                3 => {
                    self.platform_mac = Some(decoder.read_bool()?);
                }
                4 => {
                    self.platform_linux = Some(decoder.read_bool()?);
                }
                5 => {
                    self.vr_content = Some(decoder.read_bool()?);
                }
                6 => {
                    self.adult_content_violence = Some(decoder.read_bool()?);
                }
                7 => {
                    self.adult_content_sex = Some(decoder.read_bool()?);
                }
                8 => {
                    self.timestamp_updated = Some(decoder.read_varint()? as u32);
                }
                9 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.tagids_curated,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                10 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.tagids_filtered,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                11 => {
                    self.website_title = Some(decoder.read_string()?.to_owned());
                }
                12 => {
                    self.website_url = Some(decoder.read_string()?.to_owned());
                }
                13 => {
                    self.discussion_url = Some(decoder.read_string()?.to_owned());
                }
                14 => {
                    self.show_broadcast = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.platform_windows {
            encoder.write_bool_field(2, *value);
        }
        if let Some(value) = &self.platform_mac {
            encoder.write_bool_field(3, *value);
        }
        if let Some(value) = &self.platform_linux {
            encoder.write_bool_field(4, *value);
        }
        if let Some(value) = &self.vr_content {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.adult_content_violence {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.adult_content_sex {
            encoder.write_bool_field(7, *value);
        }
        if let Some(value) = &self.timestamp_updated {
            encoder.write_varint_field(8, u64::from(*value));
        }
        for value in &self.tagids_curated {
            encoder.write_varint_field(9, u64::from(*value));
        }
        for value in &self.tagids_filtered {
            encoder.write_varint_field(10, u64::from(*value));
        }
        if let Some(value) = &self.website_title {
            encoder.write_string_field(11, value);
        }
        if let Some(value) = &self.website_url {
            encoder.write_string_field(12, value);
        }
        if let Some(value) = &self.discussion_url {
            encoder.write_string_field(13, value);
        }
        if let Some(value) = &self.show_broadcast {
            encoder.write_bool_field(14, *value);
        }
    }
}

/// `CLocalizationToken` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CLocalizationToken {
    /// Field 1.
    pub language: Option<u32>,
    /// Field 2.
    pub localized_string: Option<String>,
}

impl Message for CLocalizationToken {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.language = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.localized_string = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.language {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.localized_string {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CClanEventUserNewsTuple` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CClanEventUserNewsTuple {
    /// Field 1.
    pub clanid: Option<u32>,
    /// Field 2.
    pub event_gid: Option<u64>,
    /// Field 3.
    pub announcement_gid: Option<u64>,
    /// Field 4.
    pub rtime_start: Option<u32>,
    /// Field 5.
    pub rtime_end: Option<u32>,
    /// Field 6.
    pub priority_score: Option<u32>,
    /// Field 7.
    pub r#type: Option<u32>,
    /// Field 8.
    pub clamp_range_slot: Option<u32>,
    /// Field 9.
    pub appid: Option<u32>,
    /// Field 10.
    pub rtime32_last_modified: Option<u32>,
}

impl Message for CClanEventUserNewsTuple {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.clanid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.event_gid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.announcement_gid = Some(decoder.read_fixed64()?);
                }
                4 => {
                    self.rtime_start = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.rtime_end = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.priority_score = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.r#type = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.clamp_range_slot = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.rtime32_last_modified = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.clanid {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.event_gid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.announcement_gid {
            encoder.write_fixed64_field(3, *value);
        }
        if let Some(value) = &self.rtime_start {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.rtime_end {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.priority_score {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.r#type {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.clamp_range_slot {
            encoder.write_varint_field(8, u64::from(*value));
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(9, u64::from(*value));
        }
        if let Some(value) = &self.rtime32_last_modified {
            encoder.write_varint_field(10, u64::from(*value));
        }
    }
}

/// `CClanMatchEventByRange` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CClanMatchEventByRange {
    /// Field 1.
    pub rtime_before: Option<u32>,
    /// Field 2.
    pub rtime_after: Option<u32>,
    /// Field 3.
    pub qualified: Option<u32>,
    /// Field 4.
    pub events: Vec<crate::steammessages_base::CClanEventUserNewsTuple>,
}

impl Message for CClanMatchEventByRange {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.rtime_before = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.rtime_after = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.qualified = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.events.push({
                        let mut nested =
                            crate::steammessages_base::CClanEventUserNewsTuple::default();
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
        if let Some(value) = &self.rtime_before {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.rtime_after {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.qualified {
            encoder.write_varint_field(3, u64::from(*value));
        }
        for value in &self.events {
            encoder.write_message_field(4, value);
        }
    }
}

/// `CCommunity_ClanAnnouncementInfo` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCommunity_ClanAnnouncementInfo {
    /// Field 1.
    pub gid: Option<u64>,
    /// Field 2.
    pub clanid: Option<u64>,
    /// Field 3.
    pub posterid: Option<u64>,
    /// Field 4.
    pub headline: Option<String>,
    /// Field 5.
    pub posttime: Option<u32>,
    /// Field 6.
    pub updatetime: Option<u32>,
    /// Field 7.
    pub body: Option<String>,
    /// Field 8.
    pub commentcount: Option<i32>,
    /// Field 9.
    pub tags: Vec<String>,
    /// Field 10.
    pub language: Option<i32>,
    /// Field 11.
    pub hidden: Option<bool>,
    /// Field 12.
    pub forum_topic_id: Option<u64>,
    /// Field 13.
    pub event_gid: Option<u64>,
    /// Field 14.
    pub voteupcount: Option<i32>,
    /// Field 15.
    pub votedowncount: Option<i32>,
    /// Field 16.
    pub ban_check_result: Option<crate::steammessages_base::EBanContentCheckResult>,
    /// Field 17.
    pub banned: Option<bool>,
}

impl CCommunity_ClanAnnouncementInfo {
    /// Field 16 , or its schema default when absent.
    #[must_use]
    pub fn ban_check_result_or_default(&self) -> crate::steammessages_base::EBanContentCheckResult {
        self.ban_check_result.unwrap_or(
            crate::steammessages_base::EBanContentCheckResult::k_EBanContentCheckResult_NotScanned,
        )
    }
}

impl Message for CCommunity_ClanAnnouncementInfo {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.gid = Some(decoder.read_varint()?);
                }
                2 => {
                    self.clanid = Some(decoder.read_varint()?);
                }
                3 => {
                    self.posterid = Some(decoder.read_varint()?);
                }
                4 => {
                    self.headline = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.posttime = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.updatetime = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.body = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.commentcount = Some(decoder.read_varint()? as i32);
                }
                9 => {
                    self.tags.push(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.language = Some(decoder.read_varint()? as i32);
                }
                11 => {
                    self.hidden = Some(decoder.read_bool()?);
                }
                12 => {
                    self.forum_topic_id = Some(decoder.read_fixed64()?);
                }
                13 => {
                    self.event_gid = Some(decoder.read_fixed64()?);
                }
                14 => {
                    self.voteupcount = Some(decoder.read_varint()? as i32);
                }
                15 => {
                    self.votedowncount = Some(decoder.read_varint()? as i32);
                }
                16 => {
                    self.ban_check_result =
                        Some(crate::steammessages_base::EBanContentCheckResult::from(
                            decoder.read_varint()? as i32,
                        ));
                }
                17 => {
                    self.banned = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.gid {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.clanid {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.posterid {
            encoder.write_varint_field(3, *value);
        }
        if let Some(value) = &self.headline {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.posttime {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.updatetime {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.body {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.commentcount {
            encoder.write_int32_field(8, *value);
        }
        for value in &self.tags {
            encoder.write_string_field(9, value);
        }
        if let Some(value) = &self.language {
            encoder.write_int32_field(10, *value);
        }
        if let Some(value) = &self.hidden {
            encoder.write_bool_field(11, *value);
        }
        if let Some(value) = &self.forum_topic_id {
            encoder.write_fixed64_field(12, *value);
        }
        if let Some(value) = &self.event_gid {
            encoder.write_fixed64_field(13, *value);
        }
        if let Some(value) = &self.voteupcount {
            encoder.write_int32_field(14, *value);
        }
        if let Some(value) = &self.votedowncount {
            encoder.write_int32_field(15, *value);
        }
        if let Some(value) = &self.ban_check_result {
            encoder.write_varint_field(16, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.banned {
            encoder.write_bool_field(17, *value);
        }
    }
}

/// `CClanEventData` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CClanEventData {
    /// Field 1.
    pub gid: Option<u64>,
    /// Field 2.
    pub clan_steamid: Option<u64>,
    /// Field 3.
    pub event_name: Option<String>,
    /// Field 4.
    pub event_type: Option<crate::steammessages_base::EProtoClanEventType>,
    /// Field 5.
    pub appid: Option<u32>,
    /// Field 6.
    pub server_address: Option<String>,
    /// Field 7.
    pub server_password: Option<String>,
    /// Field 8.
    pub rtime32_start_time: Option<u32>,
    /// Field 9.
    pub rtime32_end_time: Option<u32>,
    /// Field 10.
    pub comment_count: Option<i32>,
    /// Field 11.
    pub creator_steamid: Option<u64>,
    /// Field 12.
    pub last_update_steamid: Option<u64>,
    /// Field 13.
    pub event_notes: Option<String>,
    /// Field 14.
    pub jsondata: Option<String>,
    /// Field 15.
    pub announcement_body: Option<crate::steammessages_base::CCommunity_ClanAnnouncementInfo>,
    /// Field 16.
    pub published: Option<bool>,
    /// Field 17.
    pub hidden: Option<bool>,
    /// Field 18.
    pub rtime32_visibility_start: Option<u32>,
    /// Field 19.
    pub rtime32_visibility_end: Option<u32>,
    /// Field 20.
    pub broadcaster_accountid: Option<u32>,
    /// Field 21.
    pub follower_count: Option<u32>,
    /// Field 22.
    pub ignore_count: Option<u32>,
    /// Field 23.
    pub forum_topic_id: Option<u64>,
    /// Field 24.
    pub rtime32_last_modified: Option<u32>,
    /// Field 25.
    pub news_post_gid: Option<u64>,
    /// Field 26.
    pub rtime_mod_reviewed: Option<u32>,
    /// Field 27.
    pub featured_app_tagid: Option<u32>,
    /// Field 28.
    pub referenced_appids: Vec<u32>,
    /// Field 29.
    pub build_id: Option<u32>,
    /// Field 30.
    pub build_branch: Option<String>,
    /// Field 31.
    pub unlisted: Option<bool>,
    /// Field 32.
    pub rtime_created: Option<u32>,
}

impl CClanEventData {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn event_type_or_default(&self) -> crate::steammessages_base::EProtoClanEventType {
        self.event_type
            .unwrap_or(crate::steammessages_base::EProtoClanEventType::k_EClanOtherEvent)
    }
}

impl Message for CClanEventData {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.gid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.clan_steamid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.event_name = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.event_type = Some(crate::steammessages_base::EProtoClanEventType::from(
                        decoder.read_varint()? as i32,
                    ));
                }
                5 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.server_address = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.server_password = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.rtime32_start_time = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.rtime32_end_time = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.comment_count = Some(decoder.read_varint()? as i32);
                }
                11 => {
                    self.creator_steamid = Some(decoder.read_fixed64()?);
                }
                12 => {
                    self.last_update_steamid = Some(decoder.read_fixed64()?);
                }
                13 => {
                    self.event_notes = Some(decoder.read_string()?.to_owned());
                }
                14 => {
                    self.jsondata = Some(decoder.read_string()?.to_owned());
                }
                15 => {
                    self.announcement_body = Some({
                        let mut nested =
                            crate::steammessages_base::CCommunity_ClanAnnouncementInfo::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                16 => {
                    self.published = Some(decoder.read_bool()?);
                }
                17 => {
                    self.hidden = Some(decoder.read_bool()?);
                }
                18 => {
                    self.rtime32_visibility_start = Some(decoder.read_varint()? as u32);
                }
                19 => {
                    self.rtime32_visibility_end = Some(decoder.read_varint()? as u32);
                }
                20 => {
                    self.broadcaster_accountid = Some(decoder.read_varint()? as u32);
                }
                21 => {
                    self.follower_count = Some(decoder.read_varint()? as u32);
                }
                22 => {
                    self.ignore_count = Some(decoder.read_varint()? as u32);
                }
                23 => {
                    self.forum_topic_id = Some(decoder.read_fixed64()?);
                }
                24 => {
                    self.rtime32_last_modified = Some(decoder.read_varint()? as u32);
                }
                25 => {
                    self.news_post_gid = Some(decoder.read_fixed64()?);
                }
                26 => {
                    self.rtime_mod_reviewed = Some(decoder.read_varint()? as u32);
                }
                27 => {
                    self.featured_app_tagid = Some(decoder.read_varint()? as u32);
                }
                28 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.referenced_appids,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                29 => {
                    self.build_id = Some(decoder.read_varint()? as u32);
                }
                30 => {
                    self.build_branch = Some(decoder.read_string()?.to_owned());
                }
                31 => {
                    self.unlisted = Some(decoder.read_bool()?);
                }
                32 => {
                    self.rtime_created = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.clan_steamid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.event_name {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.event_type {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.server_address {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.server_password {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.rtime32_start_time {
            encoder.write_varint_field(8, u64::from(*value));
        }
        if let Some(value) = &self.rtime32_end_time {
            encoder.write_varint_field(9, u64::from(*value));
        }
        if let Some(value) = &self.comment_count {
            encoder.write_int32_field(10, *value);
        }
        if let Some(value) = &self.creator_steamid {
            encoder.write_fixed64_field(11, *value);
        }
        if let Some(value) = &self.last_update_steamid {
            encoder.write_fixed64_field(12, *value);
        }
        if let Some(value) = &self.event_notes {
            encoder.write_string_field(13, value);
        }
        if let Some(value) = &self.jsondata {
            encoder.write_string_field(14, value);
        }
        if let Some(value) = &self.announcement_body {
            encoder.write_message_field(15, value);
        }
        if let Some(value) = &self.published {
            encoder.write_bool_field(16, *value);
        }
        if let Some(value) = &self.hidden {
            encoder.write_bool_field(17, *value);
        }
        if let Some(value) = &self.rtime32_visibility_start {
            encoder.write_varint_field(18, u64::from(*value));
        }
        if let Some(value) = &self.rtime32_visibility_end {
            encoder.write_varint_field(19, u64::from(*value));
        }
        if let Some(value) = &self.broadcaster_accountid {
            encoder.write_varint_field(20, u64::from(*value));
        }
        if let Some(value) = &self.follower_count {
            encoder.write_varint_field(21, u64::from(*value));
        }
        if let Some(value) = &self.ignore_count {
            encoder.write_varint_field(22, u64::from(*value));
        }
        if let Some(value) = &self.forum_topic_id {
            encoder.write_fixed64_field(23, *value);
        }
        if let Some(value) = &self.rtime32_last_modified {
            encoder.write_varint_field(24, u64::from(*value));
        }
        if let Some(value) = &self.news_post_gid {
            encoder.write_fixed64_field(25, *value);
        }
        if let Some(value) = &self.rtime_mod_reviewed {
            encoder.write_varint_field(26, u64::from(*value));
        }
        if let Some(value) = &self.featured_app_tagid {
            encoder.write_varint_field(27, u64::from(*value));
        }
        for value in &self.referenced_appids {
            encoder.write_varint_field(28, u64::from(*value));
        }
        if let Some(value) = &self.build_id {
            encoder.write_varint_field(29, u64::from(*value));
        }
        if let Some(value) = &self.build_branch {
            encoder.write_string_field(30, value);
        }
        if let Some(value) = &self.unlisted {
            encoder.write_bool_field(31, *value);
        }
        if let Some(value) = &self.rtime_created {
            encoder.write_varint_field(32, u64::from(*value));
        }
    }
}

/// `CBilling_Address` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CBilling_Address {
    /// Field 1.
    pub first_name: Option<String>,
    /// Field 2.
    pub last_name: Option<String>,
    /// Field 3.
    pub address1: Option<String>,
    /// Field 4.
    pub address2: Option<String>,
    /// Field 5.
    pub city: Option<String>,
    /// Field 6.
    pub us_state: Option<String>,
    /// Field 7.
    pub country_code: Option<String>,
    /// Field 8.
    pub postcode: Option<String>,
    /// Field 9.
    pub zip_plus4: Option<i32>,
    /// Field 10.
    pub phone: Option<String>,
}

impl Message for CBilling_Address {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.first_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.last_name = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.address1 = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.address2 = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.city = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.us_state = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.country_code = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.postcode = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.zip_plus4 = Some(decoder.read_varint()? as i32);
                }
                10 => {
                    self.phone = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.first_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.last_name {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.address1 {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.address2 {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.city {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.us_state {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.country_code {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.postcode {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.zip_plus4 {
            encoder.write_int32_field(9, *value);
        }
        if let Some(value) = &self.phone {
            encoder.write_string_field(10, value);
        }
    }
}

/// `CPackageReservationStatus` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CPackageReservationStatus {
    /// Field 1.
    pub packageid: Option<u32>,
    /// Field 2.
    pub reservation_state: Option<i32>,
    /// Field 3.
    pub queue_position: Option<i32>,
    /// Field 4.
    pub total_queue_size: Option<i32>,
    /// Field 5.
    pub reservation_country_code: Option<String>,
    /// Field 6.
    pub expired: Option<bool>,
    /// Field 7.
    pub time_expires: Option<u32>,
    /// Field 8.
    pub time_reserved: Option<u32>,
    /// Field 9.
    pub rtime_estimated_notification: Option<u32>,
    /// Field 10.
    pub notificaton_token: Option<String>,
    /// Field 11.
    pub queue_head_position_at_reservation: Option<i32>,
    /// Field 12.
    pub queue_head_position_now: Option<i32>,
    /// Field 13.
    pub position_is_waitlist: Option<bool>,
    /// Field 14.
    pub user_waitlist_token: Option<String>,
    /// Field 15.
    pub queue_in_waitlist: Option<bool>,
    /// Field 16.
    pub queue_waitlist_token: Option<String>,
    /// Field 17.
    pub collection_time_active: Option<u32>,
}

impl Message for CPackageReservationStatus {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.packageid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.reservation_state = Some(decoder.read_varint()? as i32);
                }
                3 => {
                    self.queue_position = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.total_queue_size = Some(decoder.read_varint()? as i32);
                }
                5 => {
                    self.reservation_country_code = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.expired = Some(decoder.read_bool()?);
                }
                7 => {
                    self.time_expires = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.time_reserved = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.rtime_estimated_notification = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.notificaton_token = Some(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.queue_head_position_at_reservation = Some(decoder.read_varint()? as i32);
                }
                12 => {
                    self.queue_head_position_now = Some(decoder.read_varint()? as i32);
                }
                13 => {
                    self.position_is_waitlist = Some(decoder.read_bool()?);
                }
                14 => {
                    self.user_waitlist_token = Some(decoder.read_string()?.to_owned());
                }
                15 => {
                    self.queue_in_waitlist = Some(decoder.read_bool()?);
                }
                16 => {
                    self.queue_waitlist_token = Some(decoder.read_string()?.to_owned());
                }
                17 => {
                    self.collection_time_active = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.reservation_state {
            encoder.write_int32_field(2, *value);
        }
        if let Some(value) = &self.queue_position {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.total_queue_size {
            encoder.write_int32_field(4, *value);
        }
        if let Some(value) = &self.reservation_country_code {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.expired {
            encoder.write_bool_field(6, *value);
        }
        if let Some(value) = &self.time_expires {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.time_reserved {
            encoder.write_varint_field(8, u64::from(*value));
        }
        if let Some(value) = &self.rtime_estimated_notification {
            encoder.write_varint_field(9, u64::from(*value));
        }
        if let Some(value) = &self.notificaton_token {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.queue_head_position_at_reservation {
            encoder.write_int32_field(11, *value);
        }
        if let Some(value) = &self.queue_head_position_now {
            encoder.write_int32_field(12, *value);
        }
        if let Some(value) = &self.position_is_waitlist {
            encoder.write_bool_field(13, *value);
        }
        if let Some(value) = &self.user_waitlist_token {
            encoder.write_string_field(14, value);
        }
        if let Some(value) = &self.queue_in_waitlist {
            encoder.write_bool_field(15, *value);
        }
        if let Some(value) = &self.queue_waitlist_token {
            encoder.write_string_field(16, value);
        }
        if let Some(value) = &self.collection_time_active {
            encoder.write_varint_field(17, u64::from(*value));
        }
    }
}

/// `CMsgKeyValuePair` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgKeyValuePair {
    /// Field 1.
    pub name: Option<String>,
    /// Field 2.
    pub value: Option<String>,
}

impl Message for CMsgKeyValuePair {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.name = Some(decoder.read_string()?.to_owned());
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
        if let Some(value) = &self.name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.value {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CMsgKeyValueSet` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CMsgKeyValueSet {
    /// Field 1.
    pub pairs: Vec<crate::steammessages_base::CMsgKeyValuePair>,
}

impl Message for CMsgKeyValueSet {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.pairs.push({
                        let mut nested = crate::steammessages_base::CMsgKeyValuePair::default();
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
        for value in &self.pairs {
            encoder.write_message_field(1, value);
        }
    }
}

/// Types nested inside [`UserContentDescriptorPreferences`].
pub mod user_content_descriptor_preferences {
    use super::*;

    /// `ContentDescriptor` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct ContentDescriptor {
        /// Field 1.
        pub content_descriptorid: Option<u32>,
        /// Field 2.
        pub timestamp_added: Option<u32>,
    }

    impl Message for ContentDescriptor {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.content_descriptorid = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.timestamp_added = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.content_descriptorid {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.timestamp_added {
                encoder.write_varint_field(2, u64::from(*value));
            }
        }
    }
}

/// `UserContentDescriptorPreferences` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UserContentDescriptorPreferences {
    /// Field 1.
    pub content_descriptors_to_exclude:
        Vec<crate::steammessages_base::user_content_descriptor_preferences::ContentDescriptor>,
}

impl Message for UserContentDescriptorPreferences {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.content_descriptors_to_exclude.push({ let mut nested = crate::steammessages_base::user_content_descriptor_preferences::ContentDescriptor::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.content_descriptors_to_exclude {
            encoder.write_message_field(1, value);
        }
    }
}

/// `UserSystemInformation` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UserSystemInformation {
    /// Field 1.
    pub manufacturer: Option<String>,
    /// Field 2.
    pub model: Option<String>,
    /// Field 3.
    pub dx_video_card: Option<String>,
    /// Field 4.
    pub dx_vendorid: Option<i32>,
    /// Field 5.
    pub dx_deviceid: Option<i32>,
    /// Field 6.
    pub num_gpu: Option<u32>,
    /// Field 7.
    pub system_ram: Option<u64>,
    /// Field 8.
    pub os: Option<String>,
    /// Field 9.
    pub cpu_vendor: Option<String>,
    /// Field 10.
    pub cpu_name: Option<String>,
    /// Field 11.
    pub gaming_device_type: Option<u32>,
    /// Field 12.
    pub dx_driver_version: Option<String>,
    /// Field 14.
    pub adapter_description: Option<String>,
    /// Field 15.
    pub driver_version: Option<String>,
    /// Field 16.
    pub driver_date: Option<String>,
    /// Field 17.
    pub vram_size: Option<u32>,
    /// Field 18.
    pub screen_width: Option<u32>,
    /// Field 19.
    pub screen_height: Option<u32>,
    /// Field 20.
    pub precise_frame_rate: Option<bool>,
}

impl Message for UserSystemInformation {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.manufacturer = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.model = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.dx_video_card = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.dx_vendorid = Some(decoder.read_varint()? as i32);
                }
                5 => {
                    self.dx_deviceid = Some(decoder.read_varint()? as i32);
                }
                6 => {
                    self.num_gpu = Some(decoder.read_varint()? as u32);
                }
                7 => {
                    self.system_ram = Some(decoder.read_varint()?);
                }
                8 => {
                    self.os = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.cpu_vendor = Some(decoder.read_string()?.to_owned());
                }
                10 => {
                    self.cpu_name = Some(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.gaming_device_type = Some(decoder.read_varint()? as u32);
                }
                12 => {
                    self.dx_driver_version = Some(decoder.read_string()?.to_owned());
                }
                14 => {
                    self.adapter_description = Some(decoder.read_string()?.to_owned());
                }
                15 => {
                    self.driver_version = Some(decoder.read_string()?.to_owned());
                }
                16 => {
                    self.driver_date = Some(decoder.read_string()?.to_owned());
                }
                17 => {
                    self.vram_size = Some(decoder.read_varint()? as u32);
                }
                18 => {
                    self.screen_width = Some(decoder.read_varint()? as u32);
                }
                19 => {
                    self.screen_height = Some(decoder.read_varint()? as u32);
                }
                20 => {
                    self.precise_frame_rate = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.manufacturer {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.model {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.dx_video_card {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.dx_vendorid {
            encoder.write_int32_field(4, *value);
        }
        if let Some(value) = &self.dx_deviceid {
            encoder.write_int32_field(5, *value);
        }
        if let Some(value) = &self.num_gpu {
            encoder.write_varint_field(6, u64::from(*value));
        }
        if let Some(value) = &self.system_ram {
            encoder.write_varint_field(7, *value);
        }
        if let Some(value) = &self.os {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.cpu_vendor {
            encoder.write_string_field(9, value);
        }
        if let Some(value) = &self.cpu_name {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.gaming_device_type {
            encoder.write_varint_field(11, u64::from(*value));
        }
        if let Some(value) = &self.dx_driver_version {
            encoder.write_string_field(12, value);
        }
        if let Some(value) = &self.adapter_description {
            encoder.write_string_field(14, value);
        }
        if let Some(value) = &self.driver_version {
            encoder.write_string_field(15, value);
        }
        if let Some(value) = &self.driver_date {
            encoder.write_string_field(16, value);
        }
        if let Some(value) = &self.vram_size {
            encoder.write_varint_field(17, u64::from(*value));
        }
        if let Some(value) = &self.screen_width {
            encoder.write_varint_field(18, u64::from(*value));
        }
        if let Some(value) = &self.screen_height {
            encoder.write_varint_field(19, u64::from(*value));
        }
        if let Some(value) = &self.precise_frame_rate {
            encoder.write_bool_field(20, *value);
        }
    }
}

/// Types nested inside [`GamePerformanceSettings`].
pub mod game_performance_settings {
    use super::*;

    /// `EGamePerformanceSetting`, as a newtype so an unrecognised value round-trips instead of
    /// being rejected. Valve adds values without warning.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[repr(transparent)]
    pub struct EGamePerformanceSetting(pub i32);

    impl EGamePerformanceSetting {
        /// `k_EGamePerformanceSetting_NotSet` = `0`
        pub const k_EGamePerformanceSetting_NotSet: Self = Self(0);
        /// `k_EGamePerformanceSetting_Low` = `1`
        pub const k_EGamePerformanceSetting_Low: Self = Self(1);
        /// `k_EGamePerformanceSetting_Medium` = `2`
        pub const k_EGamePerformanceSetting_Medium: Self = Self(2);
        /// `k_EGamePerformanceSetting_High` = `3`
        pub const k_EGamePerformanceSetting_High: Self = Self(3);
        /// `k_EGamePerformanceSetting_Ultra` = `4`
        pub const k_EGamePerformanceSetting_Ultra: Self = Self(4);
        /// `k_EGamePerformanceSetting_Custom` = `5`
        pub const k_EGamePerformanceSetting_Custom: Self = Self(5);
        /// The underlying value, as it appears on the wire.
        #[must_use]
        pub const fn value(self) -> i32 {
            self.0
        }
    }

    impl From<i32> for EGamePerformanceSetting {
        fn from(value: i32) -> Self {
            Self(value)
        }
    }
}

/// `GamePerformanceSettings` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GamePerformanceSettings {
    /// Field 1.
    pub setting:
        Option<crate::steammessages_base::game_performance_settings::EGamePerformanceSetting>,
    /// Field 2.
    pub game_resolution_width: Option<u32>,
    /// Field 3.
    pub game_resolution_height: Option<u32>,
}

impl GamePerformanceSettings {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn setting_or_default(
        &self,
    ) -> crate::steammessages_base::game_performance_settings::EGamePerformanceSetting {
        self.setting.unwrap_or(crate::steammessages_base::game_performance_settings::EGamePerformanceSetting::k_EGamePerformanceSetting_NotSet)
    }
}

impl Message for GamePerformanceSettings {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.setting = Some(crate::steammessages_base::game_performance_settings::EGamePerformanceSetting::from(decoder.read_varint()? as i32));
                }
                2 => {
                    self.game_resolution_width = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.game_resolution_height = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.setting {
            encoder.write_varint_field(1, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.game_resolution_width {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.game_resolution_height {
            encoder.write_varint_field(3, u64::from(*value));
        }
    }
}
