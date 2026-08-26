//! Generated from `steammessages_auth.steamclient.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EAuthTokenPlatformType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthTokenPlatformType(pub i32);

impl EAuthTokenPlatformType {
    /// `k_EAuthTokenPlatformType_Unknown` = `0`
    pub const k_EAuthTokenPlatformType_Unknown: Self = Self(0);
    /// `k_EAuthTokenPlatformType_SteamClient` = `1`
    pub const k_EAuthTokenPlatformType_SteamClient: Self = Self(1);
    /// `k_EAuthTokenPlatformType_WebBrowser` = `2`
    pub const k_EAuthTokenPlatformType_WebBrowser: Self = Self(2);
    /// `k_EAuthTokenPlatformType_MobileApp` = `3`
    pub const k_EAuthTokenPlatformType_MobileApp: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthTokenPlatformType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAuthTokenAppType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthTokenAppType(pub i32);

impl EAuthTokenAppType {
    /// `k_EAuthTokenAppType_Unknown` = `0`
    pub const k_EAuthTokenAppType_Unknown: Self = Self(0);
    /// `k_EAuthTokenAppType_Mobile_SteamApp` = `1`
    pub const k_EAuthTokenAppType_Mobile_SteamApp: Self = Self(1);
    /// `k_EAuthTokenAppType_Mobile_ChatApp` = `2`
    pub const k_EAuthTokenAppType_Mobile_ChatApp: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthTokenAppType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAuthSessionGuardType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthSessionGuardType(pub i32);

impl EAuthSessionGuardType {
    /// `k_EAuthSessionGuardType_Unknown` = `0`
    pub const k_EAuthSessionGuardType_Unknown: Self = Self(0);
    /// `k_EAuthSessionGuardType_None` = `1`
    pub const k_EAuthSessionGuardType_None: Self = Self(1);
    /// `k_EAuthSessionGuardType_EmailCode` = `2`
    pub const k_EAuthSessionGuardType_EmailCode: Self = Self(2);
    /// `k_EAuthSessionGuardType_DeviceCode` = `3`
    pub const k_EAuthSessionGuardType_DeviceCode: Self = Self(3);
    /// `k_EAuthSessionGuardType_DeviceConfirmation` = `4`
    pub const k_EAuthSessionGuardType_DeviceConfirmation: Self = Self(4);
    /// `k_EAuthSessionGuardType_EmailConfirmation` = `5`
    pub const k_EAuthSessionGuardType_EmailConfirmation: Self = Self(5);
    /// `k_EAuthSessionGuardType_MachineToken` = `6`
    pub const k_EAuthSessionGuardType_MachineToken: Self = Self(6);
    /// `k_EAuthSessionGuardType_LegacyMachineAuth` = `7`
    pub const k_EAuthSessionGuardType_LegacyMachineAuth: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthSessionGuardType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAuthSessionSecurityHistory`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthSessionSecurityHistory(pub i32);

impl EAuthSessionSecurityHistory {
    /// `k_EAuthSessionSecurityHistory_Invalid` = `0`
    pub const k_EAuthSessionSecurityHistory_Invalid: Self = Self(0);
    /// `k_EAuthSessionSecurityHistory_UsedPreviously` = `1`
    pub const k_EAuthSessionSecurityHistory_UsedPreviously: Self = Self(1);
    /// `k_EAuthSessionSecurityHistory_NoPriorHistory` = `2`
    pub const k_EAuthSessionSecurityHistory_NoPriorHistory: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthSessionSecurityHistory {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ETokenRenewalType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ETokenRenewalType(pub i32);

impl ETokenRenewalType {
    /// `k_ETokenRenewalType_None` = `0`
    pub const k_ETokenRenewalType_None: Self = Self(0);
    /// `k_ETokenRenewalType_Allow` = `1`
    pub const k_ETokenRenewalType_Allow: Self = Self(1);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ETokenRenewalType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAuthenticationType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthenticationType(pub i32);

impl EAuthenticationType {
    /// `k_EAuthenticationType_Unknown` = `0`
    pub const k_EAuthenticationType_Unknown: Self = Self(0);
    /// `k_EAuthenticationType_Password` = `1`
    pub const k_EAuthenticationType_Password: Self = Self(1);
    /// `k_EAuthenticationType_QR` = `2`
    pub const k_EAuthenticationType_QR: Self = Self(2);
    /// `k_EAuthenticationType_AccountCreation` = `3`
    pub const k_EAuthenticationType_AccountCreation: Self = Self(3);
    /// `k_EAuthenticationType_GuestAccount` = `4`
    pub const k_EAuthenticationType_GuestAccount: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthenticationType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAuthTokenState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthTokenState(pub i32);

impl EAuthTokenState {
    /// `k_EAuthTokenState_Invalid` = `0`
    pub const k_EAuthTokenState_Invalid: Self = Self(0);
    /// `k_EAuthTokenState_New` = `1`
    pub const k_EAuthTokenState_New: Self = Self(1);
    /// `k_EAuthTokenState_Confirmed` = `2`
    pub const k_EAuthTokenState_Confirmed: Self = Self(2);
    /// `k_EAuthTokenState_Issued` = `3`
    pub const k_EAuthTokenState_Issued: Self = Self(3);
    /// `k_EAuthTokenState_Denied` = `4`
    pub const k_EAuthTokenState_Denied: Self = Self(4);
    /// `k_EAuthTokenState_LoggedOut` = `5`
    pub const k_EAuthTokenState_LoggedOut: Self = Self(5);
    /// `k_EAuthTokenState_Consumed` = `6`
    pub const k_EAuthTokenState_Consumed: Self = Self(6);
    /// `k_EAuthTokenState_Revoked` = `99`
    pub const k_EAuthTokenState_Revoked: Self = Self(99);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthTokenState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAuthTokenRevokeAction`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAuthTokenRevokeAction(pub i32);

impl EAuthTokenRevokeAction {
    /// `k_EAuthTokenRevokeLogout` = `0`
    pub const k_EAuthTokenRevokeLogout: Self = Self(0);
    /// `k_EAuthTokenRevokePermanent` = `1`
    pub const k_EAuthTokenRevokePermanent: Self = Self(1);
    /// `k_EAuthTokenRevokeReplaced` = `2`
    pub const k_EAuthTokenRevokeReplaced: Self = Self(2);
    /// `k_EAuthTokenRevokeSupport` = `3`
    pub const k_EAuthTokenRevokeSupport: Self = Self(3);
    /// `k_EAuthTokenRevokeConsume` = `4`
    pub const k_EAuthTokenRevokeConsume: Self = Self(4);
    /// `k_EAuthTokenRevokeNonRememberedLogout` = `5`
    pub const k_EAuthTokenRevokeNonRememberedLogout: Self = Self(5);
    /// `k_EAuthTokenRevokeNonRememberedPermanent` = `6`
    pub const k_EAuthTokenRevokeNonRememberedPermanent: Self = Self(6);
    /// `k_EAuthTokenRevokeAutomatic` = `7`
    pub const k_EAuthTokenRevokeAutomatic: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAuthTokenRevokeAction {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `CAuthentication_GetPasswordRSAPublicKey_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetPasswordRSAPublicKey_Request {
    /// Field 1.
    pub account_name: Option<String>,
}

impl Message for CAuthentication_GetPasswordRSAPublicKey_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.account_name = Some(decoder.read_string()?.to_owned());
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
    }
}

/// `CAuthentication_GetPasswordRSAPublicKey_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetPasswordRSAPublicKey_Response {
    /// Field 1.
    pub publickey_mod: Option<String>,
    /// Field 2.
    pub publickey_exp: Option<String>,
    /// Field 3.
    pub timestamp: Option<u64>,
}

impl Message for CAuthentication_GetPasswordRSAPublicKey_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.publickey_mod = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.publickey_exp = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.timestamp = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.publickey_mod {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.publickey_exp {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.timestamp {
            encoder.write_varint_field(3, *value);
        }
    }
}

/// `CAuthentication_DeviceDetails` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_DeviceDetails {
    /// Field 1.
    pub device_friendly_name: Option<String>,
    /// Field 2.
    pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
    /// Field 3.
    pub os_type: Option<i32>,
    /// Field 4.
    pub gaming_device_type: Option<u32>,
    /// Field 5.
    pub client_count: Option<u32>,
    /// Field 6.
    pub machine_id: Option<Vec<u8>>,
    /// Field 7.
    pub app_type: Option<crate::steammessages_auth_steamclient::EAuthTokenAppType>,
}

impl CAuthentication_DeviceDetails {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn platform_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
        self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
    }
    /// Field 7 , or its schema default when absent.
    #[must_use]
    pub fn app_type_or_default(&self) -> crate::steammessages_auth_steamclient::EAuthTokenAppType {
        self.app_type.unwrap_or(
            crate::steammessages_auth_steamclient::EAuthTokenAppType::k_EAuthTokenAppType_Unknown,
        )
    }
}

impl Message for CAuthentication_DeviceDetails {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.device_friendly_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.platform_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                3 => {
                    self.os_type = Some(decoder.read_varint()? as i32);
                }
                4 => {
                    self.gaming_device_type = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.client_count = Some(decoder.read_varint()? as u32);
                }
                6 => {
                    self.machine_id = Some(decoder.read_bytes()?.to_vec());
                }
                7 => {
                    self.app_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenAppType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.device_friendly_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.platform_type {
            encoder.write_varint_field(2, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.os_type {
            encoder.write_int32_field(3, *value);
        }
        if let Some(value) = &self.gaming_device_type {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.client_count {
            encoder.write_varint_field(5, u64::from(*value));
        }
        if let Some(value) = &self.machine_id {
            encoder.write_bytes_field(6, value);
        }
        if let Some(value) = &self.app_type {
            encoder.write_varint_field(7, i64::from(value.value()) as u64);
        }
    }
}

/// `CAuthentication_BeginAuthSessionViaQR_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_BeginAuthSessionViaQR_Request {
    /// Field 1.
    pub device_friendly_name: Option<String>,
    /// Field 2.
    pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
    /// Field 3.
    pub device_details:
        Option<crate::steammessages_auth_steamclient::CAuthentication_DeviceDetails>,
    /// Field 4.
    pub website_id: Option<String>,
}

impl CAuthentication_BeginAuthSessionViaQR_Request {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn platform_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
        self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
    }
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn website_id_or_default(&self) -> &str {
        self.website_id.as_deref().unwrap_or("Unknown")
    }
}

impl Message for CAuthentication_BeginAuthSessionViaQR_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.device_friendly_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.platform_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                3 => {
                    self.device_details = Some({
                        let mut nested = crate::steammessages_auth_steamclient::CAuthentication_DeviceDetails::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                4 => {
                    self.website_id = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.device_friendly_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.platform_type {
            encoder.write_varint_field(2, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.device_details {
            encoder.write_message_field(3, value);
        }
        if let Some(value) = &self.website_id {
            encoder.write_string_field(4, value);
        }
    }
}

/// `CAuthentication_AllowedConfirmation` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_AllowedConfirmation {
    /// Field 1.
    pub confirmation_type: Option<crate::steammessages_auth_steamclient::EAuthSessionGuardType>,
    /// Field 2.
    pub associated_message: Option<String>,
}

impl CAuthentication_AllowedConfirmation {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn confirmation_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthSessionGuardType {
        self.confirmation_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthSessionGuardType::k_EAuthSessionGuardType_Unknown)
    }
}

impl Message for CAuthentication_AllowedConfirmation {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.confirmation_type = Some(
                        crate::steammessages_auth_steamclient::EAuthSessionGuardType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                2 => {
                    self.associated_message = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.confirmation_type {
            encoder.write_varint_field(1, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.associated_message {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CAuthentication_BeginAuthSessionViaQR_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_BeginAuthSessionViaQR_Response {
    /// Field 1.
    pub client_id: Option<u64>,
    /// Field 2.
    pub challenge_url: Option<String>,
    /// Field 3.
    pub request_id: Option<Vec<u8>>,
    /// Field 4.
    pub interval: Option<f32>,
    /// Field 5.
    pub allowed_confirmations:
        Vec<crate::steammessages_auth_steamclient::CAuthentication_AllowedConfirmation>,
    /// Field 6.
    pub version: Option<i32>,
}

impl Message for CAuthentication_BeginAuthSessionViaQR_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.challenge_url = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.request_id = Some(decoder.read_bytes()?.to_vec());
                }
                4 => {
                    self.interval = Some(decoder.read_float()?);
                }
                5 => {
                    self.allowed_confirmations.push({ let mut nested = crate::steammessages_auth_steamclient::CAuthentication_AllowedConfirmation::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                6 => {
                    self.version = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.challenge_url {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.request_id {
            encoder.write_bytes_field(3, value);
        }
        if let Some(value) = &self.interval {
            encoder.write_float_field(4, *value);
        }
        for value in &self.allowed_confirmations {
            encoder.write_message_field(5, value);
        }
        if let Some(value) = &self.version {
            encoder.write_int32_field(6, *value);
        }
    }
}

/// `CAuthentication_BeginAuthSessionViaCredentials_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_BeginAuthSessionViaCredentials_Request {
    /// Field 1.
    pub device_friendly_name: Option<String>,
    /// Field 2.
    pub account_name: Option<String>,
    /// Field 3.
    pub encrypted_password: Option<String>,
    /// Field 4.
    pub encryption_timestamp: Option<u64>,
    /// Field 5.
    pub remember_login: Option<bool>,
    /// Field 6.
    pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
    /// Field 7.
    pub persistence: Option<crate::enums::ESessionPersistence>,
    /// Field 8.
    pub website_id: Option<String>,
    /// Field 9.
    pub device_details:
        Option<crate::steammessages_auth_steamclient::CAuthentication_DeviceDetails>,
    /// Field 10.
    pub guard_data: Option<String>,
    /// Field 11.
    pub language: Option<u32>,
    /// Field 12.
    pub qos_level: Option<i32>,
}

impl CAuthentication_BeginAuthSessionViaCredentials_Request {
    /// Field 6 , or its schema default when absent.
    #[must_use]
    pub fn platform_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
        self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
    }
    /// Field 7 , or its schema default when absent.
    #[must_use]
    pub fn persistence_or_default(&self) -> crate::enums::ESessionPersistence {
        self.persistence
            .unwrap_or(crate::enums::ESessionPersistence::k_ESessionPersistence_Persistent)
    }
    /// Field 8 , or its schema default when absent.
    #[must_use]
    pub fn website_id_or_default(&self) -> &str {
        self.website_id.as_deref().unwrap_or("Unknown")
    }
    /// Field 12 , or its schema default when absent.
    #[must_use]
    pub fn qos_level_or_default(&self) -> i32 {
        self.qos_level.unwrap_or(2_i32)
    }
}

impl Message for CAuthentication_BeginAuthSessionViaCredentials_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.device_friendly_name = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.account_name = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.encrypted_password = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.encryption_timestamp = Some(decoder.read_varint()?);
                }
                5 => {
                    self.remember_login = Some(decoder.read_bool()?);
                }
                6 => {
                    self.platform_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                7 => {
                    self.persistence = Some(crate::enums::ESessionPersistence::from(
                        decoder.read_varint()? as i32,
                    ));
                }
                8 => {
                    self.website_id = Some(decoder.read_string()?.to_owned());
                }
                9 => {
                    self.device_details = Some({
                        let mut nested = crate::steammessages_auth_steamclient::CAuthentication_DeviceDetails::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                10 => {
                    self.guard_data = Some(decoder.read_string()?.to_owned());
                }
                11 => {
                    self.language = Some(decoder.read_varint()? as u32);
                }
                12 => {
                    self.qos_level = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.device_friendly_name {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.account_name {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.encrypted_password {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.encryption_timestamp {
            encoder.write_varint_field(4, *value);
        }
        if let Some(value) = &self.remember_login {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.platform_type {
            encoder.write_varint_field(6, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.persistence {
            encoder.write_varint_field(7, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.website_id {
            encoder.write_string_field(8, value);
        }
        if let Some(value) = &self.device_details {
            encoder.write_message_field(9, value);
        }
        if let Some(value) = &self.guard_data {
            encoder.write_string_field(10, value);
        }
        if let Some(value) = &self.language {
            encoder.write_varint_field(11, u64::from(*value));
        }
        if let Some(value) = &self.qos_level {
            encoder.write_int32_field(12, *value);
        }
    }
}

/// `CAuthentication_BeginAuthSessionViaCredentials_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_BeginAuthSessionViaCredentials_Response {
    /// Field 1.
    pub client_id: Option<u64>,
    /// Field 2.
    pub request_id: Option<Vec<u8>>,
    /// Field 3.
    pub interval: Option<f32>,
    /// Field 4.
    pub allowed_confirmations:
        Vec<crate::steammessages_auth_steamclient::CAuthentication_AllowedConfirmation>,
    /// Field 5.
    pub steamid: Option<u64>,
    /// Field 6.
    pub weak_token: Option<String>,
    /// Field 7.
    pub agreement_session_url: Option<String>,
    /// Field 8.
    pub extended_error_message: Option<String>,
}

impl Message for CAuthentication_BeginAuthSessionViaCredentials_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.request_id = Some(decoder.read_bytes()?.to_vec());
                }
                3 => {
                    self.interval = Some(decoder.read_float()?);
                }
                4 => {
                    self.allowed_confirmations.push({ let mut nested = crate::steammessages_auth_steamclient::CAuthentication_AllowedConfirmation::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                5 => {
                    self.steamid = Some(decoder.read_varint()?);
                }
                6 => {
                    self.weak_token = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.agreement_session_url = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.extended_error_message = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.request_id {
            encoder.write_bytes_field(2, value);
        }
        if let Some(value) = &self.interval {
            encoder.write_float_field(3, *value);
        }
        for value in &self.allowed_confirmations {
            encoder.write_message_field(4, value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_varint_field(5, *value);
        }
        if let Some(value) = &self.weak_token {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.agreement_session_url {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.extended_error_message {
            encoder.write_string_field(8, value);
        }
    }
}

/// `CAuthentication_PollAuthSessionStatus_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_PollAuthSessionStatus_Request {
    /// Field 1.
    pub client_id: Option<u64>,
    /// Field 2.
    pub request_id: Option<Vec<u8>>,
    /// Field 3.
    pub token_to_revoke: Option<u64>,
}

impl Message for CAuthentication_PollAuthSessionStatus_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.request_id = Some(decoder.read_bytes()?.to_vec());
                }
                3 => {
                    self.token_to_revoke = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.request_id {
            encoder.write_bytes_field(2, value);
        }
        if let Some(value) = &self.token_to_revoke {
            encoder.write_fixed64_field(3, *value);
        }
    }
}

/// `CAuthentication_PollAuthSessionStatus_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_PollAuthSessionStatus_Response {
    /// Field 1.
    pub new_client_id: Option<u64>,
    /// Field 2.
    pub new_challenge_url: Option<String>,
    /// Field 3.
    pub refresh_token: Option<String>,
    /// Field 4.
    pub access_token: Option<String>,
    /// Field 5.
    pub had_remote_interaction: Option<bool>,
    /// Field 6.
    pub account_name: Option<String>,
    /// Field 7.
    pub new_guard_data: Option<String>,
    /// Field 8.
    pub agreement_session_url: Option<String>,
}

impl Message for CAuthentication_PollAuthSessionStatus_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.new_client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.new_challenge_url = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.refresh_token = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.access_token = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.had_remote_interaction = Some(decoder.read_bool()?);
                }
                6 => {
                    self.account_name = Some(decoder.read_string()?.to_owned());
                }
                7 => {
                    self.new_guard_data = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.agreement_session_url = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.new_client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.new_challenge_url {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.refresh_token {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.access_token {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.had_remote_interaction {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.account_name {
            encoder.write_string_field(6, value);
        }
        if let Some(value) = &self.new_guard_data {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.agreement_session_url {
            encoder.write_string_field(8, value);
        }
    }
}

/// `CAuthentication_GetAuthSessionInfo_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetAuthSessionInfo_Request {
    /// Field 1.
    pub client_id: Option<u64>,
}

impl Message for CAuthentication_GetAuthSessionInfo_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CAuthentication_GetAuthSessionInfo_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetAuthSessionInfo_Response {
    /// Field 1.
    pub ip: Option<String>,
    /// Field 2.
    pub geoloc: Option<String>,
    /// Field 3.
    pub city: Option<String>,
    /// Field 4.
    pub state: Option<String>,
    /// Field 5.
    pub country: Option<String>,
    /// Field 6.
    pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
    /// Field 7.
    pub device_friendly_name: Option<String>,
    /// Field 8.
    pub version: Option<i32>,
    /// Field 9.
    pub login_history: Option<crate::steammessages_auth_steamclient::EAuthSessionSecurityHistory>,
    /// Field 10.
    pub requestor_location_mismatch: Option<bool>,
    /// Field 11.
    pub high_usage_login: Option<bool>,
    /// Field 12.
    pub requested_persistence: Option<crate::enums::ESessionPersistence>,
    /// Field 13.
    pub device_trust: Option<i32>,
    /// Field 14.
    pub app_type: Option<crate::steammessages_auth_steamclient::EAuthTokenAppType>,
}

impl CAuthentication_GetAuthSessionInfo_Response {
    /// Field 6 , or its schema default when absent.
    #[must_use]
    pub fn platform_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
        self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
    }
    /// Field 9 , or its schema default when absent.
    #[must_use]
    pub fn login_history_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthSessionSecurityHistory {
        self.login_history.unwrap_or(crate::steammessages_auth_steamclient::EAuthSessionSecurityHistory::k_EAuthSessionSecurityHistory_Invalid)
    }
    /// Field 12 , or its schema default when absent.
    #[must_use]
    pub fn requested_persistence_or_default(&self) -> crate::enums::ESessionPersistence {
        self.requested_persistence
            .unwrap_or(crate::enums::ESessionPersistence::k_ESessionPersistence_Invalid)
    }
    /// Field 14 , or its schema default when absent.
    #[must_use]
    pub fn app_type_or_default(&self) -> crate::steammessages_auth_steamclient::EAuthTokenAppType {
        self.app_type.unwrap_or(
            crate::steammessages_auth_steamclient::EAuthTokenAppType::k_EAuthTokenAppType_Unknown,
        )
    }
}

impl Message for CAuthentication_GetAuthSessionInfo_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.ip = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.geoloc = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.city = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.state = Some(decoder.read_string()?.to_owned());
                }
                5 => {
                    self.country = Some(decoder.read_string()?.to_owned());
                }
                6 => {
                    self.platform_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                7 => {
                    self.device_friendly_name = Some(decoder.read_string()?.to_owned());
                }
                8 => {
                    self.version = Some(decoder.read_varint()? as i32);
                }
                9 => {
                    self.login_history = Some(
                        crate::steammessages_auth_steamclient::EAuthSessionSecurityHistory::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                10 => {
                    self.requestor_location_mismatch = Some(decoder.read_bool()?);
                }
                11 => {
                    self.high_usage_login = Some(decoder.read_bool()?);
                }
                12 => {
                    self.requested_persistence = Some(crate::enums::ESessionPersistence::from(
                        decoder.read_varint()? as i32,
                    ));
                }
                13 => {
                    self.device_trust = Some(decoder.read_varint()? as i32);
                }
                14 => {
                    self.app_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenAppType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.ip {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.geoloc {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.city {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.state {
            encoder.write_string_field(4, value);
        }
        if let Some(value) = &self.country {
            encoder.write_string_field(5, value);
        }
        if let Some(value) = &self.platform_type {
            encoder.write_varint_field(6, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.device_friendly_name {
            encoder.write_string_field(7, value);
        }
        if let Some(value) = &self.version {
            encoder.write_int32_field(8, *value);
        }
        if let Some(value) = &self.login_history {
            encoder.write_varint_field(9, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.requestor_location_mismatch {
            encoder.write_bool_field(10, *value);
        }
        if let Some(value) = &self.high_usage_login {
            encoder.write_bool_field(11, *value);
        }
        if let Some(value) = &self.requested_persistence {
            encoder.write_varint_field(12, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.device_trust {
            encoder.write_int32_field(13, *value);
        }
        if let Some(value) = &self.app_type {
            encoder.write_varint_field(14, i64::from(value.value()) as u64);
        }
    }
}

/// `CAuthentication_GetAuthSessionRiskInfo_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetAuthSessionRiskInfo_Request {
    /// Field 1.
    pub client_id: Option<u64>,
    /// Field 2.
    pub language: Option<u32>,
}

impl Message for CAuthentication_GetAuthSessionRiskInfo_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.language = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.language {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CAuthentication_GetAuthSessionRiskInfo_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetAuthSessionRiskInfo_Response {
    /// Field 1.
    pub location_confirmer: Option<String>,
    /// Field 2.
    pub location_requestor: Option<String>,
    /// Field 3.
    pub location_other: Option<String>,
    /// Field 4.
    pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
}

impl CAuthentication_GetAuthSessionRiskInfo_Response {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn platform_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
        self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
    }
}

impl Message for CAuthentication_GetAuthSessionRiskInfo_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.location_confirmer = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.location_requestor = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.location_other = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.platform_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.location_confirmer {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.location_requestor {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.location_other {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.platform_type {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
    }
}

/// Types nested inside [`CAuthentication_NotifyRiskQuizResults_Notification`].
pub mod c_authentication_notify_risk_quiz_results_notification {
    use super::*;

    /// `RiskQuizResults` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct RiskQuizResults {
        /// Field 1.
        pub platform: Option<bool>,
        /// Field 2.
        pub location: Option<bool>,
        /// Field 3.
        pub action: Option<bool>,
    }

    impl Message for RiskQuizResults {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.platform = Some(decoder.read_bool()?);
                    }
                    2 => {
                        self.location = Some(decoder.read_bool()?);
                    }
                    3 => {
                        self.action = Some(decoder.read_bool()?);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.platform {
                encoder.write_bool_field(1, *value);
            }
            if let Some(value) = &self.location {
                encoder.write_bool_field(2, *value);
            }
            if let Some(value) = &self.action {
                encoder.write_bool_field(3, *value);
            }
        }
    }
}

/// `CAuthentication_NotifyRiskQuizResults_Notification` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_NotifyRiskQuizResults_Notification {
    /// Field 1.
    pub client_id: Option<u64>,
    /// Field 2.
    pub results: Option<crate::steammessages_auth_steamclient::c_authentication_notify_risk_quiz_results_notification::RiskQuizResults>,
    /// Field 3.
    pub selected_action: Option<String>,
    /// Field 4.
    pub did_confirm_login: Option<bool>,
}

impl Message for CAuthentication_NotifyRiskQuizResults_Notification {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.results = Some({
                        let mut nested = crate::steammessages_auth_steamclient::c_authentication_notify_risk_quiz_results_notification::RiskQuizResults::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                3 => {
                    self.selected_action = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.did_confirm_login = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.results {
            encoder.write_message_field(2, value);
        }
        if let Some(value) = &self.selected_action {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.did_confirm_login {
            encoder.write_bool_field(4, *value);
        }
    }
}

/// `CAuthentication_UpdateAuthSessionWithMobileConfirmation_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_UpdateAuthSessionWithMobileConfirmation_Request {
    /// Field 1.
    pub version: Option<i32>,
    /// Field 2.
    pub client_id: Option<u64>,
    /// Field 3.
    pub steamid: Option<u64>,
    /// Field 4.
    pub signature: Option<Vec<u8>>,
    /// Field 5.
    pub confirm: Option<bool>,
    /// Field 6.
    pub persistence: Option<crate::enums::ESessionPersistence>,
}

impl CAuthentication_UpdateAuthSessionWithMobileConfirmation_Request {
    /// Field 5 , or its schema default when absent.
    #[must_use]
    pub fn confirm_or_default(&self) -> bool {
        self.confirm.unwrap_or(false)
    }
    /// Field 6 , or its schema default when absent.
    #[must_use]
    pub fn persistence_or_default(&self) -> crate::enums::ESessionPersistence {
        self.persistence
            .unwrap_or(crate::enums::ESessionPersistence::k_ESessionPersistence_Persistent)
    }
}

impl Message for CAuthentication_UpdateAuthSessionWithMobileConfirmation_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.version = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                3 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                4 => {
                    self.signature = Some(decoder.read_bytes()?.to_vec());
                }
                5 => {
                    self.confirm = Some(decoder.read_bool()?);
                }
                6 => {
                    self.persistence = Some(crate::enums::ESessionPersistence::from(
                        decoder.read_varint()? as i32,
                    ));
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.version {
            encoder.write_int32_field(1, *value);
        }
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(3, *value);
        }
        if let Some(value) = &self.signature {
            encoder.write_bytes_field(4, value);
        }
        if let Some(value) = &self.confirm {
            encoder.write_bool_field(5, *value);
        }
        if let Some(value) = &self.persistence {
            encoder.write_varint_field(6, i64::from(value.value()) as u64);
        }
    }
}

/// `CAuthentication_UpdateAuthSessionWithMobileConfirmation_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_UpdateAuthSessionWithMobileConfirmation_Response {}

impl Message for CAuthentication_UpdateAuthSessionWithMobileConfirmation_Response {
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

/// `CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request {
    /// Field 1.
    pub client_id: Option<u64>,
    /// Field 2.
    pub steamid: Option<u64>,
    /// Field 3.
    pub code: Option<String>,
    /// Field 4.
    pub code_type: Option<crate::steammessages_auth_steamclient::EAuthSessionGuardType>,
}

impl CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn code_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthSessionGuardType {
        self.code_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthSessionGuardType::k_EAuthSessionGuardType_Unknown)
    }
}

impl Message for CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.client_id = Some(decoder.read_varint()?);
                }
                2 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.code = Some(decoder.read_string()?.to_owned());
                }
                4 => {
                    self.code_type = Some(
                        crate::steammessages_auth_steamclient::EAuthSessionGuardType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.client_id {
            encoder.write_varint_field(1, *value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.code {
            encoder.write_string_field(3, value);
        }
        if let Some(value) = &self.code_type {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
    }
}

/// `CAuthentication_UpdateAuthSessionWithSteamGuardCode_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_UpdateAuthSessionWithSteamGuardCode_Response {
    /// Field 7.
    pub agreement_session_url: Option<String>,
}

impl Message for CAuthentication_UpdateAuthSessionWithSteamGuardCode_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                7 => {
                    self.agreement_session_url = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.agreement_session_url {
            encoder.write_string_field(7, value);
        }
    }
}

/// `CAuthentication_AccessToken_GenerateForApp_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_AccessToken_GenerateForApp_Request {
    /// Field 1.
    pub refresh_token: Option<String>,
    /// Field 2.
    pub steamid: Option<u64>,
    /// Field 3.
    pub renewal_type: Option<crate::steammessages_auth_steamclient::ETokenRenewalType>,
}

impl CAuthentication_AccessToken_GenerateForApp_Request {
    /// Field 3 , or its schema default when absent.
    #[must_use]
    pub fn renewal_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::ETokenRenewalType {
        self.renewal_type.unwrap_or(
            crate::steammessages_auth_steamclient::ETokenRenewalType::k_ETokenRenewalType_None,
        )
    }
}

impl Message for CAuthentication_AccessToken_GenerateForApp_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.refresh_token = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.renewal_type = Some(
                        crate::steammessages_auth_steamclient::ETokenRenewalType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.refresh_token {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.renewal_type {
            encoder.write_varint_field(3, i64::from(value.value()) as u64);
        }
    }
}

/// `CAuthentication_AccessToken_GenerateForApp_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_AccessToken_GenerateForApp_Response {
    /// Field 1.
    pub access_token: Option<String>,
    /// Field 2.
    pub refresh_token: Option<String>,
}

impl Message for CAuthentication_AccessToken_GenerateForApp_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.access_token = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.refresh_token = Some(decoder.read_string()?.to_owned());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.access_token {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.refresh_token {
            encoder.write_string_field(2, value);
        }
    }
}

/// `CAuthentication_RefreshToken_Enumerate_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_RefreshToken_Enumerate_Request {
    /// Field 1.
    pub include_revoked: Option<bool>,
}

impl CAuthentication_RefreshToken_Enumerate_Request {
    /// Field 1 , or its schema default when absent.
    #[must_use]
    pub fn include_revoked_or_default(&self) -> bool {
        self.include_revoked.unwrap_or(false)
    }
}

impl Message for CAuthentication_RefreshToken_Enumerate_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.include_revoked = Some(decoder.read_bool()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.include_revoked {
            encoder.write_bool_field(1, *value);
        }
    }
}

/// Types nested inside [`CAuthentication_RefreshToken_Enumerate_Response`].
pub mod c_authentication_refresh_token_enumerate_response {
    use super::*;

    /// `TokenUsageEvent` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct TokenUsageEvent {
        /// Field 1.
        pub time: Option<u32>,
        /// Field 2.
        pub ip: Option<crate::steammessages_base::CMsgIPAddress>,
        /// Field 3.
        pub locale: Option<String>,
        /// Field 4.
        pub country: Option<String>,
        /// Field 5.
        pub state: Option<String>,
        /// Field 6.
        pub city: Option<String>,
    }

    impl Message for TokenUsageEvent {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.time = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.ip = Some({
                            let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                            decoder.read_nested(|d| nested.merge(d))?;
                            nested
                        });
                    }
                    3 => {
                        self.locale = Some(decoder.read_string()?.to_owned());
                    }
                    4 => {
                        self.country = Some(decoder.read_string()?.to_owned());
                    }
                    5 => {
                        self.state = Some(decoder.read_string()?.to_owned());
                    }
                    6 => {
                        self.city = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.time {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.ip {
                encoder.write_message_field(2, value);
            }
            if let Some(value) = &self.locale {
                encoder.write_string_field(3, value);
            }
            if let Some(value) = &self.country {
                encoder.write_string_field(4, value);
            }
            if let Some(value) = &self.state {
                encoder.write_string_field(5, value);
            }
            if let Some(value) = &self.city {
                encoder.write_string_field(6, value);
            }
        }
    }

    /// `RefreshTokenDescription` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct RefreshTokenDescription {
        /// Field 1.
        pub token_id: Option<u64>,
        /// Field 2.
        pub token_description: Option<String>,
        /// Field 3.
        pub time_updated: Option<u32>,
        /// Field 4.
        pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
        /// Field 5.
        pub logged_in: Option<bool>,
        /// Field 6.
        pub os_platform: Option<u32>,
        /// Field 7.
        pub auth_type: Option<u32>,
        /// Field 8.
        pub gaming_device_type: Option<u32>,
        /// Field 9.
        pub first_seen: Option<crate::steammessages_auth_steamclient::c_authentication_refresh_token_enumerate_response::TokenUsageEvent>,
        /// Field 10.
        pub last_seen: Option<crate::steammessages_auth_steamclient::c_authentication_refresh_token_enumerate_response::TokenUsageEvent>,
        /// Field 11.
        pub os_type: Option<i32>,
        /// Field 12.
        pub authentication_type: Option<crate::steammessages_auth_steamclient::EAuthenticationType>,
        /// Field 13.
        pub effective_token_state: Option<crate::steammessages_auth_steamclient::EAuthTokenState>,
    }

    impl RefreshTokenDescription {
        /// Field 4 , or its schema default when absent.
        #[must_use]
        pub fn platform_type_or_default(
            &self,
        ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
            self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
        }
        /// Field 12 , or its schema default when absent.
        #[must_use]
        pub fn authentication_type_or_default(
            &self,
        ) -> crate::steammessages_auth_steamclient::EAuthenticationType {
            self.authentication_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthenticationType::k_EAuthenticationType_Unknown)
        }
        /// Field 13 , or its schema default when absent.
        #[must_use]
        pub fn effective_token_state_or_default(
            &self,
        ) -> crate::steammessages_auth_steamclient::EAuthTokenState {
            self.effective_token_state.unwrap_or(
                crate::steammessages_auth_steamclient::EAuthTokenState::k_EAuthTokenState_Invalid,
            )
        }
    }

    impl Message for RefreshTokenDescription {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.token_id = Some(decoder.read_fixed64()?);
                    }
                    2 => {
                        self.token_description = Some(decoder.read_string()?.to_owned());
                    }
                    3 => {
                        self.time_updated = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.platform_type = Some(
                            crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                                decoder.read_varint()? as i32,
                            ),
                        );
                    }
                    5 => {
                        self.logged_in = Some(decoder.read_bool()?);
                    }
                    6 => {
                        self.os_platform = Some(decoder.read_varint()? as u32);
                    }
                    7 => {
                        self.auth_type = Some(decoder.read_varint()? as u32);
                    }
                    8 => {
                        self.gaming_device_type = Some(decoder.read_varint()? as u32);
                    }
                    9 => {
                        self.first_seen = Some({
                            let mut nested = crate::steammessages_auth_steamclient::c_authentication_refresh_token_enumerate_response::TokenUsageEvent::default();
                            decoder.read_nested(|d| nested.merge(d))?;
                            nested
                        });
                    }
                    10 => {
                        self.last_seen = Some({
                            let mut nested = crate::steammessages_auth_steamclient::c_authentication_refresh_token_enumerate_response::TokenUsageEvent::default();
                            decoder.read_nested(|d| nested.merge(d))?;
                            nested
                        });
                    }
                    11 => {
                        self.os_type = Some(decoder.read_varint()? as i32);
                    }
                    12 => {
                        self.authentication_type = Some(
                            crate::steammessages_auth_steamclient::EAuthenticationType::from(
                                decoder.read_varint()? as i32,
                            ),
                        );
                    }
                    13 => {
                        self.effective_token_state = Some(
                            crate::steammessages_auth_steamclient::EAuthTokenState::from(
                                decoder.read_varint()? as i32,
                            ),
                        );
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.token_id {
                encoder.write_fixed64_field(1, *value);
            }
            if let Some(value) = &self.token_description {
                encoder.write_string_field(2, value);
            }
            if let Some(value) = &self.time_updated {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.platform_type {
                encoder.write_varint_field(4, i64::from(value.value()) as u64);
            }
            if let Some(value) = &self.logged_in {
                encoder.write_bool_field(5, *value);
            }
            if let Some(value) = &self.os_platform {
                encoder.write_varint_field(6, u64::from(*value));
            }
            if let Some(value) = &self.auth_type {
                encoder.write_varint_field(7, u64::from(*value));
            }
            if let Some(value) = &self.gaming_device_type {
                encoder.write_varint_field(8, u64::from(*value));
            }
            if let Some(value) = &self.first_seen {
                encoder.write_message_field(9, value);
            }
            if let Some(value) = &self.last_seen {
                encoder.write_message_field(10, value);
            }
            if let Some(value) = &self.os_type {
                encoder.write_int32_field(11, *value);
            }
            if let Some(value) = &self.authentication_type {
                encoder.write_varint_field(12, i64::from(value.value()) as u64);
            }
            if let Some(value) = &self.effective_token_state {
                encoder.write_varint_field(13, i64::from(value.value()) as u64);
            }
        }
    }
}

/// `CAuthentication_RefreshToken_Enumerate_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_RefreshToken_Enumerate_Response {
    /// Field 1.
    pub refresh_tokens: Vec<crate::steammessages_auth_steamclient::c_authentication_refresh_token_enumerate_response::RefreshTokenDescription>,
    /// Field 2.
    pub requesting_token: Option<u64>,
}

impl Message for CAuthentication_RefreshToken_Enumerate_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.refresh_tokens.push({ let mut nested = crate::steammessages_auth_steamclient::c_authentication_refresh_token_enumerate_response::RefreshTokenDescription::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.requesting_token = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.refresh_tokens {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.requesting_token {
            encoder.write_fixed64_field(2, *value);
        }
    }
}

/// `CAuthentication_GetAuthSessionsForAccount_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetAuthSessionsForAccount_Request {}

impl Message for CAuthentication_GetAuthSessionsForAccount_Request {
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

/// `CAuthentication_GetAuthSessionsForAccount_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_GetAuthSessionsForAccount_Response {
    /// Field 1.
    pub client_ids: Vec<u64>,
}

impl Message for CAuthentication_GetAuthSessionsForAccount_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.client_ids,
                    |d: &mut Decoder<'_>| d.read_varint(),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.client_ids {
            encoder.write_varint_field(1, *value);
        }
    }
}

/// `CAuthentication_Token_Revoke_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_Token_Revoke_Request {
    /// Field 1.
    pub token: Option<String>,
    /// Field 2.
    pub revoke_action: Option<crate::steammessages_auth_steamclient::EAuthTokenRevokeAction>,
}

impl CAuthentication_Token_Revoke_Request {
    /// Field 2 , or its schema default when absent.
    #[must_use]
    pub fn revoke_action_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenRevokeAction {
        self.revoke_action.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenRevokeAction::k_EAuthTokenRevokePermanent)
    }
}

impl Message for CAuthentication_Token_Revoke_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.revoke_action = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenRevokeAction::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
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
        if let Some(value) = &self.revoke_action {
            encoder.write_varint_field(2, i64::from(value.value()) as u64);
        }
    }
}

/// `CAuthentication_Token_Revoke_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_Token_Revoke_Response {}

impl Message for CAuthentication_Token_Revoke_Response {
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

/// `CAuthentication_RefreshToken_Revoke_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_RefreshToken_Revoke_Request {
    /// Field 1.
    pub token_id: Option<u64>,
    /// Field 2.
    pub steamid: Option<u64>,
    /// Field 3.
    pub revoke_action: Option<crate::steammessages_auth_steamclient::EAuthTokenRevokeAction>,
    /// Field 4.
    pub signature: Option<Vec<u8>>,
}

impl CAuthentication_RefreshToken_Revoke_Request {
    /// Field 3 , or its schema default when absent.
    #[must_use]
    pub fn revoke_action_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenRevokeAction {
        self.revoke_action.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenRevokeAction::k_EAuthTokenRevokePermanent)
    }
}

impl Message for CAuthentication_RefreshToken_Revoke_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                3 => {
                    self.revoke_action = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenRevokeAction::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                4 => {
                    self.signature = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(2, *value);
        }
        if let Some(value) = &self.revoke_action {
            encoder.write_varint_field(3, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.signature {
            encoder.write_bytes_field(4, value);
        }
    }
}

/// `CAuthentication_RefreshToken_Revoke_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthentication_RefreshToken_Revoke_Response {}

impl Message for CAuthentication_RefreshToken_Revoke_Response {
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

/// `CAuthenticationSupport_QueryRefreshTokensByAccount_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_QueryRefreshTokensByAccount_Request {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub include_revoked_tokens: Option<bool>,
}

impl Message for CAuthenticationSupport_QueryRefreshTokensByAccount_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.include_revoked_tokens = Some(decoder.read_bool()?);
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
        if let Some(value) = &self.include_revoked_tokens {
            encoder.write_bool_field(2, *value);
        }
    }
}

/// Types nested inside [`CSupportRefreshTokenDescription`].
pub mod c_support_refresh_token_description {
    use super::*;

    /// `TokenUsageEvent` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct TokenUsageEvent {
        /// Field 1.
        pub time: Option<u32>,
        /// Field 2.
        pub ip: Option<crate::steammessages_base::CMsgIPAddress>,
        /// Field 3.
        pub country: Option<String>,
        /// Field 4.
        pub state: Option<String>,
        /// Field 5.
        pub city: Option<String>,
    }

    impl Message for TokenUsageEvent {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.time = Some(decoder.read_varint()? as u32);
                    }
                    2 => {
                        self.ip = Some({
                            let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                            decoder.read_nested(|d| nested.merge(d))?;
                            nested
                        });
                    }
                    3 => {
                        self.country = Some(decoder.read_string()?.to_owned());
                    }
                    4 => {
                        self.state = Some(decoder.read_string()?.to_owned());
                    }
                    5 => {
                        self.city = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.time {
                encoder.write_varint_field(1, u64::from(*value));
            }
            if let Some(value) = &self.ip {
                encoder.write_message_field(2, value);
            }
            if let Some(value) = &self.country {
                encoder.write_string_field(3, value);
            }
            if let Some(value) = &self.state {
                encoder.write_string_field(4, value);
            }
            if let Some(value) = &self.city {
                encoder.write_string_field(5, value);
            }
        }
    }
}

/// `CSupportRefreshTokenDescription` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CSupportRefreshTokenDescription {
    /// Field 1.
    pub token_id: Option<u64>,
    /// Field 2.
    pub token_description: Option<String>,
    /// Field 3.
    pub time_updated: Option<u32>,
    /// Field 4.
    pub platform_type: Option<crate::steammessages_auth_steamclient::EAuthTokenPlatformType>,
    /// Field 5.
    pub token_state: Option<crate::steammessages_auth_steamclient::EAuthTokenState>,
    /// Field 6.
    pub owner_steamid: Option<u64>,
    /// Field 7.
    pub os_platform: Option<u32>,
    /// Field 8.
    pub os_type: Option<i32>,
    /// Field 9.
    pub auth_type: Option<u32>,
    /// Field 10.
    pub gaming_device_type: Option<u32>,
    /// Field 11.
    pub first_seen: Option<
        crate::steammessages_auth_steamclient::c_support_refresh_token_description::TokenUsageEvent,
    >,
    /// Field 12.
    pub last_seen: Option<
        crate::steammessages_auth_steamclient::c_support_refresh_token_description::TokenUsageEvent,
    >,
}

impl CSupportRefreshTokenDescription {
    /// Field 4 , or its schema default when absent.
    #[must_use]
    pub fn platform_type_or_default(
        &self,
    ) -> crate::steammessages_auth_steamclient::EAuthTokenPlatformType {
        self.platform_type.unwrap_or(crate::steammessages_auth_steamclient::EAuthTokenPlatformType::k_EAuthTokenPlatformType_Unknown)
    }
    /// Field 5 , or its schema default when absent.
    #[must_use]
    pub fn token_state_or_default(&self) -> crate::steammessages_auth_steamclient::EAuthTokenState {
        self.token_state.unwrap_or(
            crate::steammessages_auth_steamclient::EAuthTokenState::k_EAuthTokenState_Invalid,
        )
    }
}

impl Message for CSupportRefreshTokenDescription {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.token_description = Some(decoder.read_string()?.to_owned());
                }
                3 => {
                    self.time_updated = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.platform_type = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                5 => {
                    self.token_state = Some(
                        crate::steammessages_auth_steamclient::EAuthTokenState::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                6 => {
                    self.owner_steamid = Some(decoder.read_fixed64()?);
                }
                7 => {
                    self.os_platform = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.os_type = Some(decoder.read_varint()? as i32);
                }
                9 => {
                    self.auth_type = Some(decoder.read_varint()? as u32);
                }
                10 => {
                    self.gaming_device_type = Some(decoder.read_varint()? as u32);
                }
                11 => {
                    self.first_seen = Some({
                        let mut nested = crate::steammessages_auth_steamclient::c_support_refresh_token_description::TokenUsageEvent::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                12 => {
                    self.last_seen = Some({
                        let mut nested = crate::steammessages_auth_steamclient::c_support_refresh_token_description::TokenUsageEvent::default();
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
        if let Some(value) = &self.token_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.token_description {
            encoder.write_string_field(2, value);
        }
        if let Some(value) = &self.time_updated {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.platform_type {
            encoder.write_varint_field(4, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.token_state {
            encoder.write_varint_field(5, i64::from(value.value()) as u64);
        }
        if let Some(value) = &self.owner_steamid {
            encoder.write_fixed64_field(6, *value);
        }
        if let Some(value) = &self.os_platform {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.os_type {
            encoder.write_int32_field(8, *value);
        }
        if let Some(value) = &self.auth_type {
            encoder.write_varint_field(9, u64::from(*value));
        }
        if let Some(value) = &self.gaming_device_type {
            encoder.write_varint_field(10, u64::from(*value));
        }
        if let Some(value) = &self.first_seen {
            encoder.write_message_field(11, value);
        }
        if let Some(value) = &self.last_seen {
            encoder.write_message_field(12, value);
        }
    }
}

/// `CAuthenticationSupport_QueryRefreshTokensByAccount_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_QueryRefreshTokensByAccount_Response {
    /// Field 1.
    pub refresh_tokens: Vec<crate::steammessages_auth_steamclient::CSupportRefreshTokenDescription>,
    /// Field 2.
    pub last_token_reset: Option<i32>,
}

impl Message for CAuthenticationSupport_QueryRefreshTokensByAccount_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.refresh_tokens.push({ let mut nested = crate::steammessages_auth_steamclient::CSupportRefreshTokenDescription::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                2 => {
                    self.last_token_reset = Some(decoder.read_varint()? as i32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.refresh_tokens {
            encoder.write_message_field(1, value);
        }
        if let Some(value) = &self.last_token_reset {
            encoder.write_int32_field(2, *value);
        }
    }
}

/// `CAuthenticationSupport_QueryRefreshTokenByID_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_QueryRefreshTokenByID_Request {
    /// Field 1.
    pub token_id: Option<u64>,
}

impl Message for CAuthenticationSupport_QueryRefreshTokenByID_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token_id = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token_id {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// `CAuthenticationSupport_QueryRefreshTokenByID_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_QueryRefreshTokenByID_Response {
    /// Field 1.
    pub refresh_tokens: Vec<crate::steammessages_auth_steamclient::CSupportRefreshTokenDescription>,
}

impl Message for CAuthenticationSupport_QueryRefreshTokenByID_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.refresh_tokens.push({ let mut nested = crate::steammessages_auth_steamclient::CSupportRefreshTokenDescription::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.refresh_tokens {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CAuthenticationSupport_RevokeToken_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_RevokeToken_Request {
    /// Field 1.
    pub token_id: Option<u64>,
    /// Field 2.
    pub steamid: Option<u64>,
}

impl Message for CAuthenticationSupport_RevokeToken_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token_id = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token_id {
            encoder.write_fixed64_field(1, *value);
        }
        if let Some(value) = &self.steamid {
            encoder.write_fixed64_field(2, *value);
        }
    }
}

/// `CAuthenticationSupport_RevokeToken_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_RevokeToken_Response {}

impl Message for CAuthenticationSupport_RevokeToken_Response {
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

/// `CAuthenticationSupport_GetTokenHistory_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_GetTokenHistory_Request {
    /// Field 1.
    pub token_id: Option<u64>,
}

impl Message for CAuthenticationSupport_GetTokenHistory_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.token_id = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.token_id {
            encoder.write_fixed64_field(1, *value);
        }
    }
}

/// `CSupportRefreshTokenAudit` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CSupportRefreshTokenAudit {
    /// Field 1.
    pub action: Option<i32>,
    /// Field 2.
    pub time: Option<u32>,
    /// Field 3.
    pub ip: Option<crate::steammessages_base::CMsgIPAddress>,
    /// Field 4.
    pub actor: Option<u64>,
}

impl Message for CSupportRefreshTokenAudit {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.action = Some(decoder.read_varint()? as i32);
                }
                2 => {
                    self.time = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.ip = Some({
                        let mut nested = crate::steammessages_base::CMsgIPAddress::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                4 => {
                    self.actor = Some(decoder.read_fixed64()?);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.action {
            encoder.write_int32_field(1, *value);
        }
        if let Some(value) = &self.time {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.ip {
            encoder.write_message_field(3, value);
        }
        if let Some(value) = &self.actor {
            encoder.write_fixed64_field(4, *value);
        }
    }
}

/// `CAuthenticationSupport_GetTokenHistory_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_GetTokenHistory_Response {
    /// Field 1.
    pub history: Vec<crate::steammessages_auth_steamclient::CSupportRefreshTokenAudit>,
}

impl Message for CAuthenticationSupport_GetTokenHistory_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.history.push({ let mut nested = crate::steammessages_auth_steamclient::CSupportRefreshTokenAudit::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.history {
            encoder.write_message_field(1, value);
        }
    }
}

/// `CAuthenticationSupport_MarkTokenCompromised_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_MarkTokenCompromised_Request {
    /// Field 1.
    pub steamid: Option<u64>,
    /// Field 2.
    pub token_id: Option<u64>,
}

impl Message for CAuthenticationSupport_MarkTokenCompromised_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.steamid = Some(decoder.read_fixed64()?);
                }
                2 => {
                    self.token_id = Some(decoder.read_fixed64()?);
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
        if let Some(value) = &self.token_id {
            encoder.write_fixed64_field(2, *value);
        }
    }
}

/// `CAuthenticationSupport_MarkTokenCompromised_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CAuthenticationSupport_MarkTokenCompromised_Response {}

impl Message for CAuthenticationSupport_MarkTokenCompromised_Response {
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

/// `CCloudGaming_CreateNonce_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCloudGaming_CreateNonce_Request {
    /// Field 1.
    pub platform: Option<String>,
    /// Field 2.
    pub appid: Option<u32>,
}

impl Message for CCloudGaming_CreateNonce_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.platform = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.platform {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.appid {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CCloudGaming_CreateNonce_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCloudGaming_CreateNonce_Response {
    /// Field 1.
    pub nonce: Option<String>,
    /// Field 2.
    pub expiry: Option<u32>,
}

impl Message for CCloudGaming_CreateNonce_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.nonce = Some(decoder.read_string()?.to_owned());
                }
                2 => {
                    self.expiry = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.nonce {
            encoder.write_string_field(1, value);
        }
        if let Some(value) = &self.expiry {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CCloudGaming_GetTimeRemaining_Request` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCloudGaming_GetTimeRemaining_Request {
    /// Field 1.
    pub platform: Option<String>,
    /// Field 2.
    pub appid_list: Vec<u32>,
}

impl Message for CCloudGaming_GetTimeRemaining_Request {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.platform = Some(decoder.read_string()?.to_owned());
                }
                2 => decoder.read_maybe_packed(
                    key.wire_type,
                    &mut self.appid_list,
                    |d: &mut Decoder<'_>| d.read_varint().map(|v| v as u32),
                )?,
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.platform {
            encoder.write_string_field(1, value);
        }
        for value in &self.appid_list {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CCloudGaming_TimeRemaining` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCloudGaming_TimeRemaining {
    /// Field 1.
    pub appid: Option<u32>,
    /// Field 2.
    pub minutes_remaining: Option<u32>,
}

impl Message for CCloudGaming_TimeRemaining {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.appid = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.minutes_remaining = Some(decoder.read_varint()? as u32);
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
        if let Some(value) = &self.minutes_remaining {
            encoder.write_varint_field(2, u64::from(*value));
        }
    }
}

/// `CCloudGaming_GetTimeRemaining_Response` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CCloudGaming_GetTimeRemaining_Response {
    /// Field 2.
    pub entries: Vec<crate::steammessages_auth_steamclient::CCloudGaming_TimeRemaining>,
}

impl Message for CCloudGaming_GetTimeRemaining_Response {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                2 => {
                    self.entries.push({ let mut nested = crate::steammessages_auth_steamclient::CCloudGaming_TimeRemaining::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.entries {
            encoder.write_message_field(2, value);
        }
    }
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_GetPasswordRSAPublicKey_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_GetPasswordRSAPublicKey_Response;
    const TARGET: &'static str = "Authentication.GetPasswordRSAPublicKey";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_BeginAuthSessionViaQR_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_BeginAuthSessionViaQR_Response;
    const TARGET: &'static str = "Authentication.BeginAuthSessionViaQR";
}

impl tapline_wire::Rpc for crate::steammessages_auth_steamclient::CAuthentication_BeginAuthSessionViaCredentials_Request {
    type Response = crate::steammessages_auth_steamclient::CAuthentication_BeginAuthSessionViaCredentials_Response;
    const TARGET: &'static str = "Authentication.BeginAuthSessionViaCredentials";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_PollAuthSessionStatus_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_PollAuthSessionStatus_Response;
    const TARGET: &'static str = "Authentication.PollAuthSessionStatus";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_GetAuthSessionInfo_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_GetAuthSessionInfo_Response;
    const TARGET: &'static str = "Authentication.GetAuthSessionInfo";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_GetAuthSessionRiskInfo_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_GetAuthSessionRiskInfo_Response;
    const TARGET: &'static str = "Authentication.GetAuthSessionRiskInfo";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_NotifyRiskQuizResults_Notification
{
    type Response = crate::steammessages_unified_base_steamclient::NoResponse;
    const TARGET: &'static str = "Authentication.NotifyRiskQuizResults";
}

impl tapline_wire::Rpc for crate::steammessages_auth_steamclient::CAuthentication_UpdateAuthSessionWithMobileConfirmation_Request {
    type Response = crate::steammessages_auth_steamclient::CAuthentication_UpdateAuthSessionWithMobileConfirmation_Response;
    const TARGET: &'static str = "Authentication.UpdateAuthSessionWithMobileConfirmation";
}

impl tapline_wire::Rpc for crate::steammessages_auth_steamclient::CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request {
    type Response = crate::steammessages_auth_steamclient::CAuthentication_UpdateAuthSessionWithSteamGuardCode_Response;
    const TARGET: &'static str = "Authentication.UpdateAuthSessionWithSteamGuardCode";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_AccessToken_GenerateForApp_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_AccessToken_GenerateForApp_Response;
    const TARGET: &'static str = "Authentication.GenerateAccessTokenForApp";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_RefreshToken_Enumerate_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_RefreshToken_Enumerate_Response;
    const TARGET: &'static str = "Authentication.EnumerateTokens";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_GetAuthSessionsForAccount_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_GetAuthSessionsForAccount_Response;
    const TARGET: &'static str = "Authentication.GetAuthSessionsForAccount";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_Token_Revoke_Request
{
    type Response = crate::steammessages_auth_steamclient::CAuthentication_Token_Revoke_Response;
    const TARGET: &'static str = "Authentication.RevokeToken";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthentication_RefreshToken_Revoke_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthentication_RefreshToken_Revoke_Response;
    const TARGET: &'static str = "Authentication.RevokeRefreshToken";
}

impl tapline_wire::Rpc for crate::steammessages_auth_steamclient::CAuthenticationSupport_QueryRefreshTokensByAccount_Request {
    type Response = crate::steammessages_auth_steamclient::CAuthenticationSupport_QueryRefreshTokensByAccount_Response;
    const TARGET: &'static str = "AuthenticationSupport.QueryRefreshTokensByAccount";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthenticationSupport_QueryRefreshTokenByID_Request
{
    type Response = crate::steammessages_auth_steamclient::CAuthenticationSupport_QueryRefreshTokenByID_Response;
    const TARGET: &'static str = "AuthenticationSupport.QueryRefreshTokenByID";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthenticationSupport_RevokeToken_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthenticationSupport_RevokeToken_Response;
    const TARGET: &'static str = "AuthenticationSupport.RevokeToken";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthenticationSupport_GetTokenHistory_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthenticationSupport_GetTokenHistory_Response;
    const TARGET: &'static str = "AuthenticationSupport.GetTokenHistory";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CAuthenticationSupport_MarkTokenCompromised_Request
{
    type Response =
        crate::steammessages_auth_steamclient::CAuthenticationSupport_MarkTokenCompromised_Response;
    const TARGET: &'static str = "AuthenticationSupport.MarkTokenCompromised";
}

impl tapline_wire::Rpc for crate::steammessages_auth_steamclient::CCloudGaming_CreateNonce_Request {
    type Response = crate::steammessages_auth_steamclient::CCloudGaming_CreateNonce_Response;
    const TARGET: &'static str = "CloudGaming.CreateNonce";
}

impl tapline_wire::Rpc
    for crate::steammessages_auth_steamclient::CCloudGaming_GetTimeRemaining_Request
{
    type Response = crate::steammessages_auth_steamclient::CCloudGaming_GetTimeRemaining_Response;
    const TARGET: &'static str = "CloudGaming.GetTimeRemaining";
}
