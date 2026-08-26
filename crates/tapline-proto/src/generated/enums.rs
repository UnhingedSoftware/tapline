//! Generated from `enums.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EPublishedFileQueryType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPublishedFileQueryType(pub i32);

impl EPublishedFileQueryType {
    /// `k_PublishedFileQueryType_RankedByVote` = `0`
    pub const k_PublishedFileQueryType_RankedByVote: Self = Self(0);
    /// `k_PublishedFileQueryType_RankedByPublicationDate` = `1`
    pub const k_PublishedFileQueryType_RankedByPublicationDate: Self = Self(1);
    /// `k_PublishedFileQueryType_AcceptedForGameRankedByAcceptanceDate` = `2`
    pub const k_PublishedFileQueryType_AcceptedForGameRankedByAcceptanceDate: Self = Self(2);
    /// `k_PublishedFileQueryType_RankedByTrend` = `3`
    pub const k_PublishedFileQueryType_RankedByTrend: Self = Self(3);
    /// `k_PublishedFileQueryType_FavoritedByFriendsRankedByPublicationDate` = `4`
    pub const k_PublishedFileQueryType_FavoritedByFriendsRankedByPublicationDate: Self = Self(4);
    /// `k_PublishedFileQueryType_CreatedByFriendsRankedByPublicationDate` = `5`
    pub const k_PublishedFileQueryType_CreatedByFriendsRankedByPublicationDate: Self = Self(5);
    /// `k_PublishedFileQueryType_RankedByNumTimesReported` = `6`
    pub const k_PublishedFileQueryType_RankedByNumTimesReported: Self = Self(6);
    /// `k_PublishedFileQueryType_CreatedByFollowedUsersRankedByPublicationDate` = `7`
    pub const k_PublishedFileQueryType_CreatedByFollowedUsersRankedByPublicationDate: Self =
        Self(7);
    /// `k_PublishedFileQueryType_NotYetRated` = `8`
    pub const k_PublishedFileQueryType_NotYetRated: Self = Self(8);
    /// `k_PublishedFileQueryType_RankedByTotalUniqueSubscriptions` = `9`
    pub const k_PublishedFileQueryType_RankedByTotalUniqueSubscriptions: Self = Self(9);
    /// `k_PublishedFileQueryType_RankedByTotalVotesAsc` = `10`
    pub const k_PublishedFileQueryType_RankedByTotalVotesAsc: Self = Self(10);
    /// `k_PublishedFileQueryType_RankedByVotesUp` = `11`
    pub const k_PublishedFileQueryType_RankedByVotesUp: Self = Self(11);
    /// `k_PublishedFileQueryType_RankedByTextSearch` = `12`
    pub const k_PublishedFileQueryType_RankedByTextSearch: Self = Self(12);
    /// `k_PublishedFileQueryType_RankedByPlaytimeTrend` = `13`
    pub const k_PublishedFileQueryType_RankedByPlaytimeTrend: Self = Self(13);
    /// `k_PublishedFileQueryType_RankedByTotalPlaytime` = `14`
    pub const k_PublishedFileQueryType_RankedByTotalPlaytime: Self = Self(14);
    /// `k_PublishedFileQueryType_RankedByAveragePlaytimeTrend` = `15`
    pub const k_PublishedFileQueryType_RankedByAveragePlaytimeTrend: Self = Self(15);
    /// `k_PublishedFileQueryType_RankedByLifetimeAveragePlaytime` = `16`
    pub const k_PublishedFileQueryType_RankedByLifetimeAveragePlaytime: Self = Self(16);
    /// `k_PublishedFileQueryType_RankedByPlaytimeSessionsTrend` = `17`
    pub const k_PublishedFileQueryType_RankedByPlaytimeSessionsTrend: Self = Self(17);
    /// `k_PublishedFileQueryType_RankedByLifetimePlaytimeSessions` = `18`
    pub const k_PublishedFileQueryType_RankedByLifetimePlaytimeSessions: Self = Self(18);
    /// `k_PublishedFileQueryType_RankedByInappropriateContentRating` = `19`
    pub const k_PublishedFileQueryType_RankedByInappropriateContentRating: Self = Self(19);
    /// `k_PublishedFileQueryType_RankedByBanContentCheck` = `20`
    pub const k_PublishedFileQueryType_RankedByBanContentCheck: Self = Self(20);
    /// `k_PublishedFileQueryType_RankedByLastUpdatedDate` = `21`
    pub const k_PublishedFileQueryType_RankedByLastUpdatedDate: Self = Self(21);
    /// `k_PublishedFileQueryType_RankedByNumParentItems` = `22`
    pub const k_PublishedFileQueryType_RankedByNumParentItems: Self = Self(22);
    /// `k_PublishedFileQueryType_RankedByNumParentCollections` = `23`
    pub const k_PublishedFileQueryType_RankedByNumParentCollections: Self = Self(23);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPublishedFileQueryType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPublishedFileInappropriateProvider`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPublishedFileInappropriateProvider(pub i32);

impl EPublishedFileInappropriateProvider {
    /// `k_EPublishedFileInappropriateProvider_Invalid` = `0`
    pub const k_EPublishedFileInappropriateProvider_Invalid: Self = Self(0);
    /// `k_EPublishedFileInappropriateProvider_Google` = `1`
    pub const k_EPublishedFileInappropriateProvider_Google: Self = Self(1);
    /// `k_EPublishedFileInappropriateProvider_Amazon` = `2`
    pub const k_EPublishedFileInappropriateProvider_Amazon: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPublishedFileInappropriateProvider {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPublishedFileInappropriateResult`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPublishedFileInappropriateResult(pub i32);

impl EPublishedFileInappropriateResult {
    /// `k_EPublishedFileInappropriateResult_NotScanned` = `0`
    pub const k_EPublishedFileInappropriateResult_NotScanned: Self = Self(0);
    /// `k_EPublishedFileInappropriateResult_VeryUnlikely` = `1`
    pub const k_EPublishedFileInappropriateResult_VeryUnlikely: Self = Self(1);
    /// `k_EPublishedFileInappropriateResult_Unlikely` = `30`
    pub const k_EPublishedFileInappropriateResult_Unlikely: Self = Self(30);
    /// `k_EPublishedFileInappropriateResult_Possible` = `50`
    pub const k_EPublishedFileInappropriateResult_Possible: Self = Self(50);
    /// `k_EPublishedFileInappropriateResult_Likely` = `75`
    pub const k_EPublishedFileInappropriateResult_Likely: Self = Self(75);
    /// `k_EPublishedFileInappropriateResult_VeryLikely` = `100`
    pub const k_EPublishedFileInappropriateResult_VeryLikely: Self = Self(100);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPublishedFileInappropriateResult {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPersonaStateFlag`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPersonaStateFlag(pub i32);

impl EPersonaStateFlag {
    /// `k_EPersonaStateFlag_HasRichPresence` = `1`
    pub const k_EPersonaStateFlag_HasRichPresence: Self = Self(1);
    /// `k_EPersonaStateFlag_InJoinableGame` = `2`
    pub const k_EPersonaStateFlag_InJoinableGame: Self = Self(2);
    /// `k_EPersonaStateFlag_Golden` = `4`
    pub const k_EPersonaStateFlag_Golden: Self = Self(4);
    /// `k_EPersonaStateFlag_RemotePlayTogether` = `8`
    pub const k_EPersonaStateFlag_RemotePlayTogether: Self = Self(8);
    /// `k_EPersonaStateFlag_ClientTypeWeb` = `256`
    pub const k_EPersonaStateFlag_ClientTypeWeb: Self = Self(256);
    /// `k_EPersonaStateFlag_ClientTypeMobile` = `512`
    pub const k_EPersonaStateFlag_ClientTypeMobile: Self = Self(512);
    /// `k_EPersonaStateFlag_ClientTypeTenfoot` = `1024`
    pub const k_EPersonaStateFlag_ClientTypeTenfoot: Self = Self(1024);
    /// `k_EPersonaStateFlag_ClientTypeVR` = `2048`
    pub const k_EPersonaStateFlag_ClientTypeVR: Self = Self(2048);
    /// `k_EPersonaStateFlag_LaunchTypeGamepad` = `4096`
    pub const k_EPersonaStateFlag_LaunchTypeGamepad: Self = Self(4096);
    /// `k_EPersonaStateFlag_LaunchTypeCompatTool` = `8192`
    pub const k_EPersonaStateFlag_LaunchTypeCompatTool: Self = Self(8192);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPersonaStateFlag {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentCheckProvider`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentCheckProvider(pub i32);

impl EContentCheckProvider {
    /// `k_EContentCheckProvider_Invalid` = `0`
    pub const k_EContentCheckProvider_Invalid: Self = Self(0);
    /// `k_EContentCheckProvider_Google_DEPRECATED` = `1`
    pub const k_EContentCheckProvider_Google_DEPRECATED: Self = Self(1);
    /// `k_EContentCheckProvider_Amazon` = `2`
    pub const k_EContentCheckProvider_Amazon: Self = Self(2);
    /// `k_EContentCheckProvider_Local` = `3`
    pub const k_EContentCheckProvider_Local: Self = Self(3);
    /// `k_EContentCheckProvider_GoogleVertexAI` = `4`
    pub const k_EContentCheckProvider_GoogleVertexAI: Self = Self(4);
    /// `k_EContentCheckProvider_GoogleGemini` = `5`
    pub const k_EContentCheckProvider_GoogleGemini: Self = Self(5);
    /// `k_EContentCheckProvider_SteamLearn` = `6`
    pub const k_EContentCheckProvider_SteamLearn: Self = Self(6);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentCheckProvider {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EProfileCustomizationType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EProfileCustomizationType(pub i32);

impl EProfileCustomizationType {
    /// `k_EProfileCustomizationTypeInvalid` = `0`
    pub const k_EProfileCustomizationTypeInvalid: Self = Self(0);
    /// `k_EProfileCustomizationTypeRareAchievementShowcase` = `1`
    pub const k_EProfileCustomizationTypeRareAchievementShowcase: Self = Self(1);
    /// `k_EProfileCustomizationTypeGameCollector` = `2`
    pub const k_EProfileCustomizationTypeGameCollector: Self = Self(2);
    /// `k_EProfileCustomizationTypeItemShowcase` = `3`
    pub const k_EProfileCustomizationTypeItemShowcase: Self = Self(3);
    /// `k_EProfileCustomizationTypeTradeShowcase` = `4`
    pub const k_EProfileCustomizationTypeTradeShowcase: Self = Self(4);
    /// `k_EProfileCustomizationTypeBadges` = `5`
    pub const k_EProfileCustomizationTypeBadges: Self = Self(5);
    /// `k_EProfileCustomizationTypeFavoriteGame` = `6`
    pub const k_EProfileCustomizationTypeFavoriteGame: Self = Self(6);
    /// `k_EProfileCustomizationTypeScreenshotShowcase` = `7`
    pub const k_EProfileCustomizationTypeScreenshotShowcase: Self = Self(7);
    /// `k_EProfileCustomizationTypeCustomText` = `8`
    pub const k_EProfileCustomizationTypeCustomText: Self = Self(8);
    /// `k_EProfileCustomizationTypeFavoriteGroup` = `9`
    pub const k_EProfileCustomizationTypeFavoriteGroup: Self = Self(9);
    /// `k_EProfileCustomizationTypeRecommendation` = `10`
    pub const k_EProfileCustomizationTypeRecommendation: Self = Self(10);
    /// `k_EProfileCustomizationTypeWorkshopItem` = `11`
    pub const k_EProfileCustomizationTypeWorkshopItem: Self = Self(11);
    /// `k_EProfileCustomizationTypeMyWorkshop` = `12`
    pub const k_EProfileCustomizationTypeMyWorkshop: Self = Self(12);
    /// `k_EProfileCustomizationTypeArtworkShowcase` = `13`
    pub const k_EProfileCustomizationTypeArtworkShowcase: Self = Self(13);
    /// `k_EProfileCustomizationTypeVideoShowcase` = `14`
    pub const k_EProfileCustomizationTypeVideoShowcase: Self = Self(14);
    /// `k_EProfileCustomizationTypeGuides` = `15`
    pub const k_EProfileCustomizationTypeGuides: Self = Self(15);
    /// `k_EProfileCustomizationTypeMyGuides` = `16`
    pub const k_EProfileCustomizationTypeMyGuides: Self = Self(16);
    /// `k_EProfileCustomizationTypeAchievements` = `17`
    pub const k_EProfileCustomizationTypeAchievements: Self = Self(17);
    /// `k_EProfileCustomizationTypeGreenlight` = `18`
    pub const k_EProfileCustomizationTypeGreenlight: Self = Self(18);
    /// `k_EProfileCustomizationTypeMyGreenlight` = `19`
    pub const k_EProfileCustomizationTypeMyGreenlight: Self = Self(19);
    /// `k_EProfileCustomizationTypeSalien` = `20`
    pub const k_EProfileCustomizationTypeSalien: Self = Self(20);
    /// `k_EProfileCustomizationTypeLoyaltyRewardReactions` = `21`
    pub const k_EProfileCustomizationTypeLoyaltyRewardReactions: Self = Self(21);
    /// `k_EProfileCustomizationTypeSingleArtworkShowcase` = `22`
    pub const k_EProfileCustomizationTypeSingleArtworkShowcase: Self = Self(22);
    /// `k_EProfileCustomizationTypeAchievementsCompletionist` = `23`
    pub const k_EProfileCustomizationTypeAchievementsCompletionist: Self = Self(23);
    /// `k_EProfileCustomizationTypeReplay` = `24`
    pub const k_EProfileCustomizationTypeReplay: Self = Self(24);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EProfileCustomizationType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPublishedFileStorageSystem`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPublishedFileStorageSystem(pub i32);

impl EPublishedFileStorageSystem {
    /// `k_EPublishedFileStorageSystemInvalid` = `0`
    pub const k_EPublishedFileStorageSystemInvalid: Self = Self(0);
    /// `k_EPublishedFileStorageSystemLegacyCloud` = `1`
    pub const k_EPublishedFileStorageSystemLegacyCloud: Self = Self(1);
    /// `k_EPublishedFileStorageSystemDepot` = `2`
    pub const k_EPublishedFileStorageSystemDepot: Self = Self(2);
    /// `k_EPublishedFileStorageSystemUGCCloud` = `3`
    pub const k_EPublishedFileStorageSystemUGCCloud: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPublishedFileStorageSystem {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECloudStoragePersistState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECloudStoragePersistState(pub i32);

impl ECloudStoragePersistState {
    /// `k_ECloudStoragePersistStatePersisted` = `0`
    pub const k_ECloudStoragePersistStatePersisted: Self = Self(0);
    /// `k_ECloudStoragePersistStateForgotten` = `1`
    pub const k_ECloudStoragePersistStateForgotten: Self = Self(1);
    /// `k_ECloudStoragePersistStateDeleted` = `2`
    pub const k_ECloudStoragePersistStateDeleted: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECloudStoragePersistState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESDCardFormatStage`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESDCardFormatStage(pub i32);

impl ESDCardFormatStage {
    /// `k_ESDCardFormatStage_Invalid` = `0`
    pub const k_ESDCardFormatStage_Invalid: Self = Self(0);
    /// `k_ESDCardFormatStage_Starting` = `1`
    pub const k_ESDCardFormatStage_Starting: Self = Self(1);
    /// `k_ESDCardFormatStage_Testing` = `2`
    pub const k_ESDCardFormatStage_Testing: Self = Self(2);
    /// `k_ESDCardFormatStage_Rescuing` = `3`
    pub const k_ESDCardFormatStage_Rescuing: Self = Self(3);
    /// `k_ESDCardFormatStage_Formatting` = `4`
    pub const k_ESDCardFormatStage_Formatting: Self = Self(4);
    /// `k_ESDCardFormatStage_Finalizing` = `5`
    pub const k_ESDCardFormatStage_Finalizing: Self = Self(5);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESDCardFormatStage {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EStorageFormatStage`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EStorageFormatStage(pub i32);

impl EStorageFormatStage {
    /// `k_EStorageFormatStage_Invalid` = `0`
    pub const k_EStorageFormatStage_Invalid: Self = Self(0);
    /// `k_EStorageFormatStage_NotRunning` = `1`
    pub const k_EStorageFormatStage_NotRunning: Self = Self(1);
    /// `k_EStorageFormatStage_Starting` = `2`
    pub const k_EStorageFormatStage_Starting: Self = Self(2);
    /// `k_EStorageFormatStage_Testing` = `3`
    pub const k_EStorageFormatStage_Testing: Self = Self(3);
    /// `k_EStorageFormatStage_Rescuing` = `4`
    pub const k_EStorageFormatStage_Rescuing: Self = Self(4);
    /// `k_EStorageFormatStage_Formatting` = `5`
    pub const k_EStorageFormatStage_Formatting: Self = Self(5);
    /// `k_EStorageFormatStage_Finalizing` = `6`
    pub const k_EStorageFormatStage_Finalizing: Self = Self(6);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EStorageFormatStage {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemFanControlMode`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemFanControlMode(pub i32);

impl ESystemFanControlMode {
    /// `k_SystemFanControlMode_Invalid` = `0`
    pub const k_SystemFanControlMode_Invalid: Self = Self(0);
    /// `k_SystemFanControlMode_Disabled` = `1`
    pub const k_SystemFanControlMode_Disabled: Self = Self(1);
    /// `k_SystemFanControlMode_Default` = `2`
    pub const k_SystemFanControlMode_Default: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemFanControlMode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EColorGamutLabelSet`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EColorGamutLabelSet(pub i32);

impl EColorGamutLabelSet {
    /// `k_ColorGamutLabelSet_Default` = `0`
    pub const k_ColorGamutLabelSet_Default: Self = Self(0);
    /// `k_ColorGamutLabelSet_sRGB_Native` = `1`
    pub const k_ColorGamutLabelSet_sRGB_Native: Self = Self(1);
    /// `k_ColorGamutLabelSet_Native_sRGB_Boosted` = `2`
    pub const k_ColorGamutLabelSet_Native_sRGB_Boosted: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EColorGamutLabelSet {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EWindowStackingOrder`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EWindowStackingOrder(pub i32);

impl EWindowStackingOrder {
    /// `k_EWindowStackingOrder_Invalid` = `0`
    pub const k_EWindowStackingOrder_Invalid: Self = Self(0);
    /// `k_EWindowStackingOrder_Top` = `1`
    pub const k_EWindowStackingOrder_Top: Self = Self(1);
    /// `k_EWindowStackingOrder_Bottom` = `2`
    pub const k_EWindowStackingOrder_Bottom: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EWindowStackingOrder {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBluetoothDeviceType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBluetoothDeviceType(pub i32);

impl EBluetoothDeviceType {
    /// `k_BluetoothDeviceType_Invalid` = `0`
    pub const k_BluetoothDeviceType_Invalid: Self = Self(0);
    /// `k_BluetoothDeviceType_Unknown` = `1`
    pub const k_BluetoothDeviceType_Unknown: Self = Self(1);
    /// `k_BluetoothDeviceType_Phone` = `2`
    pub const k_BluetoothDeviceType_Phone: Self = Self(2);
    /// `k_BluetoothDeviceType_Computer` = `3`
    pub const k_BluetoothDeviceType_Computer: Self = Self(3);
    /// `k_BluetoothDeviceType_Headset` = `4`
    pub const k_BluetoothDeviceType_Headset: Self = Self(4);
    /// `k_BluetoothDeviceType_Headphones` = `5`
    pub const k_BluetoothDeviceType_Headphones: Self = Self(5);
    /// `k_BluetoothDeviceType_Speakers` = `6`
    pub const k_BluetoothDeviceType_Speakers: Self = Self(6);
    /// `k_BluetoothDeviceType_OtherAudio` = `7`
    pub const k_BluetoothDeviceType_OtherAudio: Self = Self(7);
    /// `k_BluetoothDeviceType_Mouse` = `8`
    pub const k_BluetoothDeviceType_Mouse: Self = Self(8);
    /// `k_BluetoothDeviceType_Joystick` = `9`
    pub const k_BluetoothDeviceType_Joystick: Self = Self(9);
    /// `k_BluetoothDeviceType_Gamepad` = `10`
    pub const k_BluetoothDeviceType_Gamepad: Self = Self(10);
    /// `k_BluetoothDeviceType_Keyboard` = `11`
    pub const k_BluetoothDeviceType_Keyboard: Self = Self(11);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBluetoothDeviceType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESpeakerConfiguration`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESpeakerConfiguration(pub i32);

impl ESpeakerConfiguration {
    /// `k_ESpeakerConfiguration_Other` = `0`
    pub const k_ESpeakerConfiguration_Other: Self = Self(0);
    /// `k_ESpeakerConfiguration_Stereo` = `1`
    pub const k_ESpeakerConfiguration_Stereo: Self = Self(1);
    /// `k_ESpeakerConfiguration_51` = `2`
    pub const k_ESpeakerConfiguration_51: Self = Self(2);
    /// `k_ESpeakerConfiguration_71` = `3`
    pub const k_ESpeakerConfiguration_71: Self = Self(3);
    /// `k_ESpeakerConfiguration_51_Ac3` = `4`
    pub const k_ESpeakerConfiguration_51_Ac3: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESpeakerConfiguration {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemAudioDirection`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemAudioDirection(pub i32);

impl ESystemAudioDirection {
    /// `k_SystemAudioDirection_Invalid` = `0`
    pub const k_SystemAudioDirection_Invalid: Self = Self(0);
    /// `k_SystemAudioDirection_Input` = `1`
    pub const k_SystemAudioDirection_Input: Self = Self(1);
    /// `k_SystemAudioDirection_Output` = `2`
    pub const k_SystemAudioDirection_Output: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemAudioDirection {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemAudioChannel`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemAudioChannel(pub i32);

impl ESystemAudioChannel {
    /// `k_SystemAudioChannel_Invalid` = `0`
    pub const k_SystemAudioChannel_Invalid: Self = Self(0);
    /// `k_SystemAudioChannel_Aggregated` = `1`
    pub const k_SystemAudioChannel_Aggregated: Self = Self(1);
    /// `k_SystemAudioChannel_FrontLeft` = `2`
    pub const k_SystemAudioChannel_FrontLeft: Self = Self(2);
    /// `k_SystemAudioChannel_FrontRight` = `3`
    pub const k_SystemAudioChannel_FrontRight: Self = Self(3);
    /// `k_SystemAudioChannel_LFE` = `4`
    pub const k_SystemAudioChannel_LFE: Self = Self(4);
    /// `k_SystemAudioChannel_BackLeft` = `5`
    pub const k_SystemAudioChannel_BackLeft: Self = Self(5);
    /// `k_SystemAudioChannel_BackRight` = `6`
    pub const k_SystemAudioChannel_BackRight: Self = Self(6);
    /// `k_SystemAudioChannel_FrontCenter` = `7`
    pub const k_SystemAudioChannel_FrontCenter: Self = Self(7);
    /// `k_SystemAudioChannel_Unknown` = `8`
    pub const k_SystemAudioChannel_Unknown: Self = Self(8);
    /// `k_SystemAudioChannel_Mono` = `9`
    pub const k_SystemAudioChannel_Mono: Self = Self(9);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemAudioChannel {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemAudioPortType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemAudioPortType(pub i32);

impl ESystemAudioPortType {
    /// `k_SystemAudioPortType_Invalid` = `0`
    pub const k_SystemAudioPortType_Invalid: Self = Self(0);
    /// `k_SystemAudioPortType_Unknown` = `1`
    pub const k_SystemAudioPortType_Unknown: Self = Self(1);
    /// `k_SystemAudioPortType_Audio32f` = `2`
    pub const k_SystemAudioPortType_Audio32f: Self = Self(2);
    /// `k_SystemAudioPortType_Midi8b` = `3`
    pub const k_SystemAudioPortType_Midi8b: Self = Self(3);
    /// `k_SystemAudioPortType_Video32RGBA` = `4`
    pub const k_SystemAudioPortType_Video32RGBA: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemAudioPortType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemAudioPortDirection`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemAudioPortDirection(pub i32);

impl ESystemAudioPortDirection {
    /// `k_SystemAudioPortDirection_Invalid` = `0`
    pub const k_SystemAudioPortDirection_Invalid: Self = Self(0);
    /// `k_SystemAudioPortDirection_Input` = `1`
    pub const k_SystemAudioPortDirection_Input: Self = Self(1);
    /// `k_SystemAudioPortDirection_Output` = `2`
    pub const k_SystemAudioPortDirection_Output: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemAudioPortDirection {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemServiceState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemServiceState(pub i32);

impl ESystemServiceState {
    /// `k_ESystemServiceState_Unavailable` = `0`
    pub const k_ESystemServiceState_Unavailable: Self = Self(0);
    /// `k_ESystemServiceState_Disabled` = `1`
    pub const k_ESystemServiceState_Disabled: Self = Self(1);
    /// `k_ESystemServiceState_Enabled` = `2`
    pub const k_ESystemServiceState_Enabled: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemServiceState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBluetoothAudioPreference`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBluetoothAudioPreference(pub i32);

impl EBluetoothAudioPreference {
    /// `k_EBluetoothAudioPreference_Quality` = `0`
    pub const k_EBluetoothAudioPreference_Quality: Self = Self(0);
    /// `k_EBluetoothAudioPreference_Latency` = `1`
    pub const k_EBluetoothAudioPreference_Latency: Self = Self(1);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBluetoothAudioPreference {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGraphicsPerfOverlayLevel`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGraphicsPerfOverlayLevel(pub i32);

impl EGraphicsPerfOverlayLevel {
    /// `k_EGraphicsPerfOverlayLevel_Hidden` = `0`
    pub const k_EGraphicsPerfOverlayLevel_Hidden: Self = Self(0);
    /// `k_EGraphicsPerfOverlayLevel_Basic` = `1`
    pub const k_EGraphicsPerfOverlayLevel_Basic: Self = Self(1);
    /// `k_EGraphicsPerfOverlayLevel_Medium` = `2`
    pub const k_EGraphicsPerfOverlayLevel_Medium: Self = Self(2);
    /// `k_EGraphicsPerfOverlayLevel_Full` = `3`
    pub const k_EGraphicsPerfOverlayLevel_Full: Self = Self(3);
    /// `k_EGraphicsPerfOverlayLevel_Minimal` = `4`
    pub const k_EGraphicsPerfOverlayLevel_Minimal: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGraphicsPerfOverlayLevel {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGPUPerformanceLevel`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGPUPerformanceLevel(pub i32);

impl EGPUPerformanceLevel {
    /// `k_EGPUPerformanceLevel_Invalid` = `0`
    pub const k_EGPUPerformanceLevel_Invalid: Self = Self(0);
    /// `k_EGPUPerformanceLevel_Auto` = `1`
    pub const k_EGPUPerformanceLevel_Auto: Self = Self(1);
    /// `k_EGPUPerformanceLevel_Manual` = `2`
    pub const k_EGPUPerformanceLevel_Manual: Self = Self(2);
    /// `k_EGPUPerformanceLevel_Low` = `3`
    pub const k_EGPUPerformanceLevel_Low: Self = Self(3);
    /// `k_EGPUPerformanceLevel_High` = `4`
    pub const k_EGPUPerformanceLevel_High: Self = Self(4);
    /// `k_EGPUPerformanceLevel_Profiling` = `5`
    pub const k_EGPUPerformanceLevel_Profiling: Self = Self(5);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGPUPerformanceLevel {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESplitScalingFilter`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESplitScalingFilter(pub i32);

impl ESplitScalingFilter {
    /// `k_ESplitScalingFilter_Invalid` = `0`
    pub const k_ESplitScalingFilter_Invalid: Self = Self(0);
    /// `k_ESplitScalingFilter_Linear` = `1`
    pub const k_ESplitScalingFilter_Linear: Self = Self(1);
    /// `k_ESplitScalingFilter_Nearest` = `2`
    pub const k_ESplitScalingFilter_Nearest: Self = Self(2);
    /// `k_ESplitScalingFilter_Sharp` = `3`
    pub const k_ESplitScalingFilter_Sharp: Self = Self(3);
    /// `k_ESplitScalingFilter_NIS_Deprecated` = `4`
    pub const k_ESplitScalingFilter_NIS_Deprecated: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESplitScalingFilter {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESplitScalingScaler`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESplitScalingScaler(pub i32);

impl ESplitScalingScaler {
    /// `k_ESplitScalingScaler_Invalid` = `0`
    pub const k_ESplitScalingScaler_Invalid: Self = Self(0);
    /// `k_ESplitScalingScaler_Auto` = `1`
    pub const k_ESplitScalingScaler_Auto: Self = Self(1);
    /// `k_ESplitScalingScaler_Integer` = `2`
    pub const k_ESplitScalingScaler_Integer: Self = Self(2);
    /// `k_ESplitScalingScaler_Fit` = `3`
    pub const k_ESplitScalingScaler_Fit: Self = Self(3);
    /// `k_ESplitScalingScaler_Fill` = `4`
    pub const k_ESplitScalingScaler_Fill: Self = Self(4);
    /// `k_ESplitScalingScaler_Stretch` = `5`
    pub const k_ESplitScalingScaler_Stretch: Self = Self(5);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESplitScalingScaler {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGamescopeBlurMode`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGamescopeBlurMode(pub i32);

impl EGamescopeBlurMode {
    /// `k_EGamescopeBlurMode_Disabled` = `0`
    pub const k_EGamescopeBlurMode_Disabled: Self = Self(0);
    /// `k_EGamescopeBlurMode_IfOccluded` = `1`
    pub const k_EGamescopeBlurMode_IfOccluded: Self = Self(1);
    /// `k_EGamescopeBlurMode_Always` = `2`
    pub const k_EGamescopeBlurMode_Always: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGamescopeBlurMode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESLSHelper`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESLSHelper(pub i32);

impl ESLSHelper {
    /// `k_ESLSHelper_Invalid` = `0`
    pub const k_ESLSHelper_Invalid: Self = Self(0);
    /// `k_ESLSHelper_Minidump` = `1`
    pub const k_ESLSHelper_Minidump: Self = Self(1);
    /// `k_ESLSHelper_Kdump` = `2`
    pub const k_ESLSHelper_Kdump: Self = Self(2);
    /// `k_ESLSHelper_Journal` = `3`
    pub const k_ESLSHelper_Journal: Self = Self(3);
    /// `k_ESLSHelper_Gpu` = `4`
    pub const k_ESLSHelper_Gpu: Self = Self(4);
    /// `k_ESLSHelper_SystemInfo` = `5`
    pub const k_ESLSHelper_SystemInfo: Self = Self(5);
    /// `k_ESLSHelper_Devcoredump` = `6`
    pub const k_ESLSHelper_Devcoredump: Self = Self(6);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESLSHelper {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EHDRVisualization`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EHDRVisualization(pub i32);

impl EHDRVisualization {
    /// `k_EHDRVisualization_None` = `0`
    pub const k_EHDRVisualization_None: Self = Self(0);
    /// `k_EHDRVisualization_Heatmap` = `1`
    pub const k_EHDRVisualization_Heatmap: Self = Self(1);
    /// `k_EHDRVisualization_Analysis` = `2`
    pub const k_EHDRVisualization_Analysis: Self = Self(2);
    /// `k_EHDRVisualization_HeatmapExtended` = `3`
    pub const k_EHDRVisualization_HeatmapExtended: Self = Self(3);
    /// `k_EHDRVisualization_HeatmapClassic` = `4`
    pub const k_EHDRVisualization_HeatmapClassic: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EHDRVisualization {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EHDRToneMapOperator`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EHDRToneMapOperator(pub i32);

impl EHDRToneMapOperator {
    /// `k_EHDRToneMapOperator_Invalid` = `0`
    pub const k_EHDRToneMapOperator_Invalid: Self = Self(0);
    /// `k_EHDRToneMapOperator_Uncharted` = `1`
    pub const k_EHDRToneMapOperator_Uncharted: Self = Self(1);
    /// `k_EHDRToneMapOperator_Reinhard` = `2`
    pub const k_EHDRToneMapOperator_Reinhard: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EHDRToneMapOperator {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECPUGovernor`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECPUGovernor(pub i32);

impl ECPUGovernor {
    /// `k_ECPUGovernor_Invalid` = `0`
    pub const k_ECPUGovernor_Invalid: Self = Self(0);
    /// `k_ECPUGovernor_Perf` = `1`
    pub const k_ECPUGovernor_Perf: Self = Self(1);
    /// `k_ECPUGovernor_Powersave` = `2`
    pub const k_ECPUGovernor_Powersave: Self = Self(2);
    /// `k_ECPUGovernor_Manual` = `3`
    pub const k_ECPUGovernor_Manual: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECPUGovernor {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EUpdaterType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EUpdaterType(pub i32);

impl EUpdaterType {
    /// `k_EUpdaterType_Invalid` = `0`
    pub const k_EUpdaterType_Invalid: Self = Self(0);
    /// `k_EUpdaterType_Client` = `1`
    pub const k_EUpdaterType_Client: Self = Self(1);
    /// `k_EUpdaterType_OS` = `2`
    pub const k_EUpdaterType_OS: Self = Self(2);
    /// `k_EUpdaterType_BIOS` = `3`
    pub const k_EUpdaterType_BIOS: Self = Self(3);
    /// `k_EUpdaterType_Aggregated` = `4`
    pub const k_EUpdaterType_Aggregated: Self = Self(4);
    /// `k_EUpdaterType_Test1` = `5`
    pub const k_EUpdaterType_Test1: Self = Self(5);
    /// `k_EUpdaterType_Test2` = `6`
    pub const k_EUpdaterType_Test2: Self = Self(6);
    /// `k_EUpdaterType_Dummy` = `7`
    pub const k_EUpdaterType_Dummy: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EUpdaterType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EUpdaterState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EUpdaterState(pub i32);

impl EUpdaterState {
    /// `k_EUpdaterState_Invalid` = `0`
    pub const k_EUpdaterState_Invalid: Self = Self(0);
    /// `k_EUpdaterState_UpToDate` = `2`
    pub const k_EUpdaterState_UpToDate: Self = Self(2);
    /// `k_EUpdaterState_Checking` = `3`
    pub const k_EUpdaterState_Checking: Self = Self(3);
    /// `k_EUpdaterState_Available` = `4`
    pub const k_EUpdaterState_Available: Self = Self(4);
    /// `k_EUpdaterState_Applying` = `5`
    pub const k_EUpdaterState_Applying: Self = Self(5);
    /// `k_EUpdaterState_ClientRestartPending` = `6`
    pub const k_EUpdaterState_ClientRestartPending: Self = Self(6);
    /// `k_EUpdaterState_SystemRestartPending` = `7`
    pub const k_EUpdaterState_SystemRestartPending: Self = Self(7);
    /// `k_EUpdaterState_RollBack` = `8`
    pub const k_EUpdaterState_RollBack: Self = Self(8);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EUpdaterState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EStorageBlockContentType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EStorageBlockContentType(pub i32);

impl EStorageBlockContentType {
    /// `k_EStorageBlockContentType_Invalid` = `0`
    pub const k_EStorageBlockContentType_Invalid: Self = Self(0);
    /// `k_EStorageBlockContentType_Unknown` = `1`
    pub const k_EStorageBlockContentType_Unknown: Self = Self(1);
    /// `k_EStorageBlockContentType_FileSystem` = `2`
    pub const k_EStorageBlockContentType_FileSystem: Self = Self(2);
    /// `k_EStorageBlockContentType_Crypto` = `3`
    pub const k_EStorageBlockContentType_Crypto: Self = Self(3);
    /// `k_EStorageBlockContentType_Raid` = `4`
    pub const k_EStorageBlockContentType_Raid: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EStorageBlockContentType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EStorageBlockFileSystemType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EStorageBlockFileSystemType(pub i32);

impl EStorageBlockFileSystemType {
    /// `k_EStorageBlockFileSystemType_Invalid` = `0`
    pub const k_EStorageBlockFileSystemType_Invalid: Self = Self(0);
    /// `k_EStorageBlockFileSystemType_Unknown` = `1`
    pub const k_EStorageBlockFileSystemType_Unknown: Self = Self(1);
    /// `k_EStorageBlockFileSystemType_VFat` = `2`
    pub const k_EStorageBlockFileSystemType_VFat: Self = Self(2);
    /// `k_EStorageBlockFileSystemType_Ext4` = `3`
    pub const k_EStorageBlockFileSystemType_Ext4: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EStorageBlockFileSystemType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EStorageDriveMediaType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EStorageDriveMediaType(pub i32);

impl EStorageDriveMediaType {
    /// `k_EStorageDriveMediaType_Invalid` = `0`
    pub const k_EStorageDriveMediaType_Invalid: Self = Self(0);
    /// `k_EStorageDriveMediaType_Unknown` = `1`
    pub const k_EStorageDriveMediaType_Unknown: Self = Self(1);
    /// `k_EStorageDriveMediaType_HDD` = `2`
    pub const k_EStorageDriveMediaType_HDD: Self = Self(2);
    /// `k_EStorageDriveMediaType_SSD` = `3`
    pub const k_EStorageDriveMediaType_SSD: Self = Self(3);
    /// `k_EStorageDriveMediaType_Removable` = `4`
    pub const k_EStorageDriveMediaType_Removable: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EStorageDriveMediaType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESystemDisplayCompatibilityMode`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESystemDisplayCompatibilityMode(pub i32);

impl ESystemDisplayCompatibilityMode {
    /// `k_ESystemDisplayCompatibilityMode_Invalid` = `0`
    pub const k_ESystemDisplayCompatibilityMode_Invalid: Self = Self(0);
    /// `k_ESystemDisplayCompatibilityMode_None` = `1`
    pub const k_ESystemDisplayCompatibilityMode_None: Self = Self(1);
    /// `k_ESystemDisplayCompatibilityMode_MinimalBandwith` = `2`
    pub const k_ESystemDisplayCompatibilityMode_MinimalBandwith: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESystemDisplayCompatibilityMode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESteamOSCompatibilityCategory`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESteamOSCompatibilityCategory(pub i32);

impl ESteamOSCompatibilityCategory {
    /// `k_ESteamOSCompatibilityCategory_Unknown` = `0`
    pub const k_ESteamOSCompatibilityCategory_Unknown: Self = Self(0);
    /// `k_ESteamOSCompatibilityCategory_Unsupported` = `1`
    pub const k_ESteamOSCompatibilityCategory_Unsupported: Self = Self(1);
    /// `k_ESteamOSCompatibilityCategory_Compatible` = `2`
    pub const k_ESteamOSCompatibilityCategory_Compatible: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESteamOSCompatibilityCategory {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESteamOSCompatibilityResultDisplayType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESteamOSCompatibilityResultDisplayType(pub i32);

impl ESteamOSCompatibilityResultDisplayType {
    /// `k_ESteamOSCompatibilityResultDisplayType_Invisible` = `0`
    pub const k_ESteamOSCompatibilityResultDisplayType_Invisible: Self = Self(0);
    /// `k_ESteamOSCompatibilityResultDisplayType_Informational` = `1`
    pub const k_ESteamOSCompatibilityResultDisplayType_Informational: Self = Self(1);
    /// `k_ESteamOSCompatibilityResultDisplayType_Unsupported` = `2`
    pub const k_ESteamOSCompatibilityResultDisplayType_Unsupported: Self = Self(2);
    /// `k_ESteamOSCompatibilityResultDisplayType_Compatible` = `3`
    pub const k_ESteamOSCompatibilityResultDisplayType_Compatible: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESteamOSCompatibilityResultDisplayType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESteamDeckCompatibilityCategory`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESteamDeckCompatibilityCategory(pub i32);

impl ESteamDeckCompatibilityCategory {
    /// `k_ESteamDeckCompatibilityCategory_Unknown` = `0`
    pub const k_ESteamDeckCompatibilityCategory_Unknown: Self = Self(0);
    /// `k_ESteamDeckCompatibilityCategory_Unsupported` = `1`
    pub const k_ESteamDeckCompatibilityCategory_Unsupported: Self = Self(1);
    /// `k_ESteamDeckCompatibilityCategory_Playable` = `2`
    pub const k_ESteamDeckCompatibilityCategory_Playable: Self = Self(2);
    /// `k_ESteamDeckCompatibilityCategory_Verified` = `3`
    pub const k_ESteamDeckCompatibilityCategory_Verified: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESteamDeckCompatibilityCategory {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESteamDeckCompatibilityResultDisplayType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESteamDeckCompatibilityResultDisplayType(pub i32);

impl ESteamDeckCompatibilityResultDisplayType {
    /// `k_ESteamDeckCompatibilityResultDisplayType_Invisible` = `0`
    pub const k_ESteamDeckCompatibilityResultDisplayType_Invisible: Self = Self(0);
    /// `k_ESteamDeckCompatibilityResultDisplayType_Informational` = `1`
    pub const k_ESteamDeckCompatibilityResultDisplayType_Informational: Self = Self(1);
    /// `k_ESteamDeckCompatibilityResultDisplayType_Unsupported` = `2`
    pub const k_ESteamDeckCompatibilityResultDisplayType_Unsupported: Self = Self(2);
    /// `k_ESteamDeckCompatibilityResultDisplayType_Playable` = `3`
    pub const k_ESteamDeckCompatibilityResultDisplayType_Playable: Self = Self(3);
    /// `k_ESteamDeckCompatibilityResultDisplayType_Verified` = `4`
    pub const k_ESteamDeckCompatibilityResultDisplayType_Verified: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESteamDeckCompatibilityResultDisplayType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESteamDeckCompatibilityTestResult`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESteamDeckCompatibilityTestResult(pub i32);

impl ESteamDeckCompatibilityTestResult {
    /// `k_ESteamDeckCompatibilityTestResult_Invalid` = `0`
    pub const k_ESteamDeckCompatibilityTestResult_Invalid: Self = Self(0);
    /// `k_ESteamDeckCompatibilityTestResult_NotApplicable` = `1`
    pub const k_ESteamDeckCompatibilityTestResult_NotApplicable: Self = Self(1);
    /// `k_ESteamDeckCompatibilityTestResult_Pass` = `2`
    pub const k_ESteamDeckCompatibilityTestResult_Pass: Self = Self(2);
    /// `k_ESteamDeckCompatibilityTestResult_Fail` = `3`
    pub const k_ESteamDeckCompatibilityTestResult_Fail: Self = Self(3);
    /// `k_ESteamDeckCompatibilityTestResult_FailMinor` = `4`
    pub const k_ESteamDeckCompatibilityTestResult_FailMinor: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESteamDeckCompatibilityTestResult {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EACState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EACState(pub i32);

impl EACState {
    /// `k_EACState_Unknown` = `0`
    pub const k_EACState_Unknown: Self = Self(0);
    /// `k_EACState_Disconnected` = `1`
    pub const k_EACState_Disconnected: Self = Self(1);
    /// `k_EACState_Connected` = `2`
    pub const k_EACState_Connected: Self = Self(2);
    /// `k_EACState_ConnectedSlow` = `3`
    pub const k_EACState_ConnectedSlow: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EACState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBatteryState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBatteryState(pub i32);

impl EBatteryState {
    /// `k_EBatteryState_Unknown` = `0`
    pub const k_EBatteryState_Unknown: Self = Self(0);
    /// `k_EBatteryState_Discharging` = `1`
    pub const k_EBatteryState_Discharging: Self = Self(1);
    /// `k_EBatteryState_Charging` = `2`
    pub const k_EBatteryState_Charging: Self = Self(2);
    /// `k_EBatteryState_Full` = `3`
    pub const k_EBatteryState_Full: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBatteryState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EOSBranch`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EOSBranch(pub i32);

impl EOSBranch {
    /// `k_EOSBranch_Unknown` = `0`
    pub const k_EOSBranch_Unknown: Self = Self(0);
    /// `k_EOSBranch_Release` = `1`
    pub const k_EOSBranch_Release: Self = Self(1);
    /// `k_EOSBranch_ReleaseCandidate` = `2`
    pub const k_EOSBranch_ReleaseCandidate: Self = Self(2);
    /// `k_EOSBranch_Beta` = `3`
    pub const k_EOSBranch_Beta: Self = Self(3);
    /// `k_EOSBranch_BetaCandidate` = `4`
    pub const k_EOSBranch_BetaCandidate: Self = Self(4);
    /// `k_EOSBranch_Preview` = `5`
    pub const k_EOSBranch_Preview: Self = Self(5);
    /// `k_EOSBranch_PreviewCandidate` = `6`
    pub const k_EOSBranch_PreviewCandidate: Self = Self(6);
    /// `k_EOSBranch_Main` = `7`
    pub const k_EOSBranch_Main: Self = Self(7);
    /// `k_EOSBranch_Staging` = `8`
    pub const k_EOSBranch_Staging: Self = Self(8);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EOSBranch {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBrowserGPUStatus`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBrowserGPUStatus(pub i32);

impl EBrowserGPUStatus {
    /// `k_EBrowserGPUStatus_Invalid` = `0`
    pub const k_EBrowserGPUStatus_Invalid: Self = Self(0);
    /// `k_EBrowserGPUStatus_Enabled` = `1`
    pub const k_EBrowserGPUStatus_Enabled: Self = Self(1);
    /// `k_EBrowserGPUStatus_DisabledUnknown` = `2`
    pub const k_EBrowserGPUStatus_DisabledUnknown: Self = Self(2);
    /// `k_EBrowserGPUStatus_DisabledCrashCount` = `4`
    pub const k_EBrowserGPUStatus_DisabledCrashCount: Self = Self(4);
    /// `k_EBrowserGPUStatus_DisabledBlocklist` = `5`
    pub const k_EBrowserGPUStatus_DisabledBlocklist: Self = Self(5);
    /// `k_EBrowserGPUStatus_DisabledJSRequest` = `6`
    pub const k_EBrowserGPUStatus_DisabledJSRequest: Self = Self(6);
    /// `k_EBrowserGPUStatus_DisabledCommandLine` = `7`
    pub const k_EBrowserGPUStatus_DisabledCommandLine: Self = Self(7);
    /// `k_EBrowserGPUStatus_DisabledRuntimeDetect` = `8`
    pub const k_EBrowserGPUStatus_DisabledRuntimeDetect: Self = Self(8);
    /// `k_EBrowserGPUStatus_DisabledChildCommandLine` = `9`
    pub const k_EBrowserGPUStatus_DisabledChildCommandLine: Self = Self(9);
    /// `k_EBrowserGPUStatus_DisabledCompositingCommandLine` = `10`
    pub const k_EBrowserGPUStatus_DisabledCompositingCommandLine: Self = Self(10);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBrowserGPUStatus {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBrowserFeatureStatus`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBrowserFeatureStatus(pub i32);

impl EBrowserFeatureStatus {
    /// `k_EBrowserFeatureStatus_Invalid` = `0`
    pub const k_EBrowserFeatureStatus_Invalid: Self = Self(0);
    /// `k_EBrowserFeatureStatus_NotFound` = `1`
    pub const k_EBrowserFeatureStatus_NotFound: Self = Self(1);
    /// `k_EBrowserFeatureStatus_Unknown` = `2`
    pub const k_EBrowserFeatureStatus_Unknown: Self = Self(2);
    /// `k_EBrowserFeatureStatus_DisabledSoftware` = `3`
    pub const k_EBrowserFeatureStatus_DisabledSoftware: Self = Self(3);
    /// `k_EBrowserFeatureStatus_DisabledOff` = `4`
    pub const k_EBrowserFeatureStatus_DisabledOff: Self = Self(4);
    /// `k_EBrowserFeatureStatus_DisabledOffOk` = `5`
    pub const k_EBrowserFeatureStatus_DisabledOffOk: Self = Self(5);
    /// `k_EBrowserFeatureStatus_UnavailableSoftware` = `6`
    pub const k_EBrowserFeatureStatus_UnavailableSoftware: Self = Self(6);
    /// `k_EBrowserFeatureStatus_UnavailableOff` = `7`
    pub const k_EBrowserFeatureStatus_UnavailableOff: Self = Self(7);
    /// `k_EBrowserFeatureStatus_UnavailableOffOk` = `8`
    pub const k_EBrowserFeatureStatus_UnavailableOffOk: Self = Self(8);
    /// `k_EBrowserFeatureStatus_EnabledReadback` = `9`
    pub const k_EBrowserFeatureStatus_EnabledReadback: Self = Self(9);
    /// `k_EBrowserFeatureStatus_EnabledForce` = `10`
    pub const k_EBrowserFeatureStatus_EnabledForce: Self = Self(10);
    /// `k_EBrowserFeatureStatus_Enabled` = `11`
    pub const k_EBrowserFeatureStatus_Enabled: Self = Self(11);
    /// `k_EBrowserFeatureStatus_EnabledOn` = `12`
    pub const k_EBrowserFeatureStatus_EnabledOn: Self = Self(12);
    /// `k_EBrowserFeatureStatus_EnabledForceOn` = `13`
    pub const k_EBrowserFeatureStatus_EnabledForceOn: Self = Self(13);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBrowserFeatureStatus {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGpuDriverId`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGpuDriverId(pub i32);

impl EGpuDriverId {
    /// `k_EGpuDriverId_Invalid` = `0`
    pub const k_EGpuDriverId_Invalid: Self = Self(0);
    /// `k_EGpuDriverId_Unknown` = `1`
    pub const k_EGpuDriverId_Unknown: Self = Self(1);
    /// `k_EGpuDriverId_AmdProprietary` = `2`
    pub const k_EGpuDriverId_AmdProprietary: Self = Self(2);
    /// `k_EGpuDriverId_AmdOpenSource` = `3`
    pub const k_EGpuDriverId_AmdOpenSource: Self = Self(3);
    /// `k_EGpuDriverId_MesaRadv` = `4`
    pub const k_EGpuDriverId_MesaRadv: Self = Self(4);
    /// `k_EGpuDriverId_NvidiaProprietary` = `5`
    pub const k_EGpuDriverId_NvidiaProprietary: Self = Self(5);
    /// `k_EGpuDriverId_IntelPropietary` = `6`
    pub const k_EGpuDriverId_IntelPropietary: Self = Self(6);
    /// `k_EGpuDriverId_MesaIntel` = `7`
    pub const k_EGpuDriverId_MesaIntel: Self = Self(7);
    /// `k_EGpuDriverId_QualcommProprietary` = `8`
    pub const k_EGpuDriverId_QualcommProprietary: Self = Self(8);
    /// `k_EGpuDriverId_ArmProprietary` = `9`
    pub const k_EGpuDriverId_ArmProprietary: Self = Self(9);
    /// `k_EGpuDriverId_GoogleSwiftshader` = `10`
    pub const k_EGpuDriverId_GoogleSwiftshader: Self = Self(10);
    /// `k_EGpuDriverId_BroadcomProprietary` = `11`
    pub const k_EGpuDriverId_BroadcomProprietary: Self = Self(11);
    /// `k_EGpuDriverId_MesaLLVMPipe` = `12`
    pub const k_EGpuDriverId_MesaLLVMPipe: Self = Self(12);
    /// `k_EGpuDriverId_MoltenVK` = `13`
    pub const k_EGpuDriverId_MoltenVK: Self = Self(13);
    /// `k_EGpuDriverId_MesaTurnip` = `14`
    pub const k_EGpuDriverId_MesaTurnip: Self = Self(14);
    /// `k_EGpuDriverId_MesaPanVK` = `15`
    pub const k_EGpuDriverId_MesaPanVK: Self = Self(15);
    /// `k_EGpuDriverId_MesaVenus` = `16`
    pub const k_EGpuDriverId_MesaVenus: Self = Self(16);
    /// `k_EGpuDriverId_MesaDozen` = `17`
    pub const k_EGpuDriverId_MesaDozen: Self = Self(17);
    /// `k_EGpuDriverId_MesaNVK` = `18`
    pub const k_EGpuDriverId_MesaNVK: Self = Self(18);
    /// `k_EGpuDriverId_MesaHoneyKrisp` = `19`
    pub const k_EGpuDriverId_MesaHoneyKrisp: Self = Self(19);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGpuDriverId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECommunityItemClass`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECommunityItemClass(pub i32);

impl ECommunityItemClass {
    /// `k_ECommunityItemClass_Invalid` = `0`
    pub const k_ECommunityItemClass_Invalid: Self = Self(0);
    /// `k_ECommunityItemClass_Badge` = `1`
    pub const k_ECommunityItemClass_Badge: Self = Self(1);
    /// `k_ECommunityItemClass_GameCard` = `2`
    pub const k_ECommunityItemClass_GameCard: Self = Self(2);
    /// `k_ECommunityItemClass_ProfileBackground` = `3`
    pub const k_ECommunityItemClass_ProfileBackground: Self = Self(3);
    /// `k_ECommunityItemClass_Emoticon` = `4`
    pub const k_ECommunityItemClass_Emoticon: Self = Self(4);
    /// `k_ECommunityItemClass_BoosterPack` = `5`
    pub const k_ECommunityItemClass_BoosterPack: Self = Self(5);
    /// `k_ECommunityItemClass_Consumable` = `6`
    pub const k_ECommunityItemClass_Consumable: Self = Self(6);
    /// `k_ECommunityItemClass_GameGoo` = `7`
    pub const k_ECommunityItemClass_GameGoo: Self = Self(7);
    /// `k_ECommunityItemClass_ProfileModifier` = `8`
    pub const k_ECommunityItemClass_ProfileModifier: Self = Self(8);
    /// `k_ECommunityItemClass_Scene` = `9`
    pub const k_ECommunityItemClass_Scene: Self = Self(9);
    /// `k_ECommunityItemClass_SalienItem` = `10`
    pub const k_ECommunityItemClass_SalienItem: Self = Self(10);
    /// `k_ECommunityItemClass_Sticker` = `11`
    pub const k_ECommunityItemClass_Sticker: Self = Self(11);
    /// `k_ECommunityItemClass_ChatEffect` = `12`
    pub const k_ECommunityItemClass_ChatEffect: Self = Self(12);
    /// `k_ECommunityItemClass_MiniProfileBackground` = `13`
    pub const k_ECommunityItemClass_MiniProfileBackground: Self = Self(13);
    /// `k_ECommunityItemClass_AvatarFrame` = `14`
    pub const k_ECommunityItemClass_AvatarFrame: Self = Self(14);
    /// `k_ECommunityItemClass_AnimatedAvatar` = `15`
    pub const k_ECommunityItemClass_AnimatedAvatar: Self = Self(15);
    /// `k_ECommunityItemClass_SteamDeckKeyboardSkin` = `16`
    pub const k_ECommunityItemClass_SteamDeckKeyboardSkin: Self = Self(16);
    /// `k_ECommunityItemClass_SteamDeckStartupMovie` = `17`
    pub const k_ECommunityItemClass_SteamDeckStartupMovie: Self = Self(17);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECommunityItemClass {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESteamDeckCompatibilityFeedback`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESteamDeckCompatibilityFeedback(pub i32);

impl ESteamDeckCompatibilityFeedback {
    /// `k_ESteamDeckCompatibilityFeedback_Unset` = `0`
    pub const k_ESteamDeckCompatibilityFeedback_Unset: Self = Self(0);
    /// `k_ESteamDeckCompatibilityFeedback_Agree` = `1`
    pub const k_ESteamDeckCompatibilityFeedback_Agree: Self = Self(1);
    /// `k_ESteamDeckCompatibilityFeedback_Disagree` = `2`
    pub const k_ESteamDeckCompatibilityFeedback_Disagree: Self = Self(2);
    /// `k_ESteamDeckCompatibilityFeedback_Ignore` = `3`
    pub const k_ESteamDeckCompatibilityFeedback_Ignore: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESteamDeckCompatibilityFeedback {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EProvideDeckFeedbackPreference`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EProvideDeckFeedbackPreference(pub i32);

impl EProvideDeckFeedbackPreference {
    /// `k_EProvideDeckFeedbackPreference_Unset` = `0`
    pub const k_EProvideDeckFeedbackPreference_Unset: Self = Self(0);
    /// `k_EProvideDeckFeedbackPreference_Yes` = `1`
    pub const k_EProvideDeckFeedbackPreference_Yes: Self = Self(1);
    /// `k_EProvideDeckFeedbackPreference_No` = `2`
    pub const k_EProvideDeckFeedbackPreference_No: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EProvideDeckFeedbackPreference {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EHardwareCompatibilityFeedbackDetails`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EHardwareCompatibilityFeedbackDetails(pub i32);

impl EHardwareCompatibilityFeedbackDetails {
    /// `k_EHardwareCompatibilityFeedbackDetails_Unset` = `0`
    pub const k_EHardwareCompatibilityFeedbackDetails_Unset: Self = Self(0);
    /// `k_EHardwareCompatibilityFeedbackDetails_Performance` = `1`
    pub const k_EHardwareCompatibilityFeedbackDetails_Performance: Self = Self(1);
    /// `k_EHardwareCompatibilityFeedbackDetails_Stability` = `2`
    pub const k_EHardwareCompatibilityFeedbackDetails_Stability: Self = Self(2);
    /// `k_EHardwareCompatibilityFeedbackDetails_Legibility` = `4`
    pub const k_EHardwareCompatibilityFeedbackDetails_Legibility: Self = Self(4);
    /// `k_EHardwareCompatibilityFeedbackDetails_Input` = `8`
    pub const k_EHardwareCompatibilityFeedbackDetails_Input: Self = Self(8);
    /// `k_EHardwareCompatibilityFeedbackDetails_Other` = `16`
    pub const k_EHardwareCompatibilityFeedbackDetails_Other: Self = Self(16);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EHardwareCompatibilityFeedbackDetails {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGameFrameRateReportingPreference`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGameFrameRateReportingPreference(pub i32);

impl EGameFrameRateReportingPreference {
    /// `k_EGameFrameRateReportingPreference_Unset` = `0`
    pub const k_EGameFrameRateReportingPreference_Unset: Self = Self(0);
    /// `k_EGameFrameRateReportingPreference_No` = `1`
    pub const k_EGameFrameRateReportingPreference_No: Self = Self(1);
    /// `k_EGameFrameRateReportingPreference_Yes_Anonymous` = `2`
    pub const k_EGameFrameRateReportingPreference_Yes_Anonymous: Self = Self(2);
    /// `k_EGameFrameRateReportingPreference_Yes_NonAnonymous` = `3`
    pub const k_EGameFrameRateReportingPreference_Yes_NonAnonymous: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGameFrameRateReportingPreference {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ETouchGesture`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ETouchGesture(pub i32);

impl ETouchGesture {
    /// `k_ETouchGestureNone` = `0`
    pub const k_ETouchGestureNone: Self = Self(0);
    /// `k_ETouchGestureTouch` = `1`
    pub const k_ETouchGestureTouch: Self = Self(1);
    /// `k_ETouchGestureTap` = `2`
    pub const k_ETouchGestureTap: Self = Self(2);
    /// `k_ETouchGestureDoubleTap` = `3`
    pub const k_ETouchGestureDoubleTap: Self = Self(3);
    /// `k_ETouchGestureShortPress` = `4`
    pub const k_ETouchGestureShortPress: Self = Self(4);
    /// `k_ETouchGestureLongPress` = `5`
    pub const k_ETouchGestureLongPress: Self = Self(5);
    /// `k_ETouchGestureLongTap` = `6`
    pub const k_ETouchGestureLongTap: Self = Self(6);
    /// `k_ETouchGestureTwoFingerTap` = `7`
    pub const k_ETouchGestureTwoFingerTap: Self = Self(7);
    /// `k_ETouchGestureTapCancelled` = `8`
    pub const k_ETouchGestureTapCancelled: Self = Self(8);
    /// `k_ETouchGesturePinchBegin` = `9`
    pub const k_ETouchGesturePinchBegin: Self = Self(9);
    /// `k_ETouchGesturePinchUpdate` = `10`
    pub const k_ETouchGesturePinchUpdate: Self = Self(10);
    /// `k_ETouchGesturePinchEnd` = `11`
    pub const k_ETouchGesturePinchEnd: Self = Self(11);
    /// `k_ETouchGestureFlingStart` = `12`
    pub const k_ETouchGestureFlingStart: Self = Self(12);
    /// `k_ETouchGestureFlingCancelled` = `13`
    pub const k_ETouchGestureFlingCancelled: Self = Self(13);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ETouchGesture {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ESessionPersistence`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ESessionPersistence(pub i32);

impl ESessionPersistence {
    /// `k_ESessionPersistence_Invalid` = `-1`
    pub const k_ESessionPersistence_Invalid: Self = Self(-1);
    /// `k_ESessionPersistence_Ephemeral` = `0`
    pub const k_ESessionPersistence_Ephemeral: Self = Self(0);
    /// `k_ESessionPersistence_Persistent` = `1`
    pub const k_ESessionPersistence_Persistent: Self = Self(1);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ESessionPersistence {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ENewSteamAnnouncementState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ENewSteamAnnouncementState(pub i32);

impl ENewSteamAnnouncementState {
    /// `k_ENewSteamAnnouncementState_Invalid` = `0`
    pub const k_ENewSteamAnnouncementState_Invalid: Self = Self(0);
    /// `k_ENewSteamAnnouncementState_AllRead` = `1`
    pub const k_ENewSteamAnnouncementState_AllRead: Self = Self(1);
    /// `k_ENewSteamAnnouncementState_NewAnnouncement` = `2`
    pub const k_ENewSteamAnnouncementState_NewAnnouncement: Self = Self(2);
    /// `k_ENewSteamAnnouncementState_FeaturedAnnouncement` = `3`
    pub const k_ENewSteamAnnouncementState_FeaturedAnnouncement: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ENewSteamAnnouncementState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EForumType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EForumType(pub i32);

impl EForumType {
    /// `k_EForumType_Invalid` = `0`
    pub const k_EForumType_Invalid: Self = Self(0);
    /// `k_EForumType_General` = `1`
    pub const k_EForumType_General: Self = Self(1);
    /// `k_EForumType_ReportedPosts` = `2`
    pub const k_EForumType_ReportedPosts: Self = Self(2);
    /// `k_EForumType_Workshop` = `3`
    pub const k_EForumType_Workshop: Self = Self(3);
    /// `k_EForumType_PublishedFile` = `4`
    pub const k_EForumType_PublishedFile: Self = Self(4);
    /// `k_EForumType_Trading` = `5`
    pub const k_EForumType_Trading: Self = Self(5);
    /// `k_EForumType_PlayTest` = `6`
    pub const k_EForumType_PlayTest: Self = Self(6);
    /// `k_EForumType_Event` = `7`
    pub const k_EForumType_Event: Self = Self(7);
    /// `k_EForumType_Max` = `8`
    pub const k_EForumType_Max: Self = Self(8);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EForumType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECommentThreadType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECommentThreadType(pub i32);

impl ECommentThreadType {
    /// `k_ECommentThreadTypeInvalid` = `0`
    pub const k_ECommentThreadTypeInvalid: Self = Self(0);
    /// `k_ECommentThreadTypeScreenshot_Deprecated` = `1`
    pub const k_ECommentThreadTypeScreenshot_Deprecated: Self = Self(1);
    /// `k_ECommentThreadTypeWorkshopAccount_Developer` = `2`
    pub const k_ECommentThreadTypeWorkshopAccount_Developer: Self = Self(2);
    /// `k_ECommentThreadTypeWorkshopAccount_Public` = `3`
    pub const k_ECommentThreadTypeWorkshopAccount_Public: Self = Self(3);
    /// `k_ECommentThreadTypePublishedFile_Developer` = `4`
    pub const k_ECommentThreadTypePublishedFile_Developer: Self = Self(4);
    /// `k_ECommentThreadTypePublishedFile_Public` = `5`
    pub const k_ECommentThreadTypePublishedFile_Public: Self = Self(5);
    /// `k_ECommentThreadTypeTest` = `6`
    pub const k_ECommentThreadTypeTest: Self = Self(6);
    /// `k_ECommentThreadTypeForumTopic` = `7`
    pub const k_ECommentThreadTypeForumTopic: Self = Self(7);
    /// `k_ECommentThreadTypeRecommendation` = `8`
    pub const k_ECommentThreadTypeRecommendation: Self = Self(8);
    /// `k_ECommentThreadTypeVideo_Deprecated` = `9`
    pub const k_ECommentThreadTypeVideo_Deprecated: Self = Self(9);
    /// `k_ECommentThreadTypeProfile` = `10`
    pub const k_ECommentThreadTypeProfile: Self = Self(10);
    /// `k_ECommentThreadTypeNewsPost` = `11`
    pub const k_ECommentThreadTypeNewsPost: Self = Self(11);
    /// `k_ECommentThreadTypeClan` = `12`
    pub const k_ECommentThreadTypeClan: Self = Self(12);
    /// `k_ECommentThreadTypeClanAnnouncement` = `13`
    pub const k_ECommentThreadTypeClanAnnouncement: Self = Self(13);
    /// `k_ECommentThreadTypeClanEvent` = `14`
    pub const k_ECommentThreadTypeClanEvent: Self = Self(14);
    /// `k_ECommentThreadTypeUserStatusPublished` = `15`
    pub const k_ECommentThreadTypeUserStatusPublished: Self = Self(15);
    /// `k_ECommentThreadTypeUserReceivedNewGame` = `16`
    pub const k_ECommentThreadTypeUserReceivedNewGame: Self = Self(16);
    /// `k_ECommentThreadTypePublishedFile_Announcement` = `17`
    pub const k_ECommentThreadTypePublishedFile_Announcement: Self = Self(17);
    /// `k_ECommentThreadTypeModeratorMessage` = `18`
    pub const k_ECommentThreadTypeModeratorMessage: Self = Self(18);
    /// `k_ECommentThreadTypeClanCuratedApp` = `19`
    pub const k_ECommentThreadTypeClanCuratedApp: Self = Self(19);
    /// `k_ECommentThreadTypeQAndASession` = `20`
    pub const k_ECommentThreadTypeQAndASession: Self = Self(20);
    /// `k_ECommentThreadTypeMax` = `21`
    pub const k_ECommentThreadTypeMax: Self = Self(21);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECommentThreadType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBroadcastPermission`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBroadcastPermission(pub i32);

impl EBroadcastPermission {
    /// `k_EBroadcastPermissionDisabled` = `0`
    pub const k_EBroadcastPermissionDisabled: Self = Self(0);
    /// `k_EBroadcastPermissionFriendsApprove` = `1`
    pub const k_EBroadcastPermissionFriendsApprove: Self = Self(1);
    /// `k_EBroadcastPermissionFriendsAllowed` = `2`
    pub const k_EBroadcastPermissionFriendsAllowed: Self = Self(2);
    /// `k_EBroadcastPermissionPublic` = `3`
    pub const k_EBroadcastPermissionPublic: Self = Self(3);
    /// `k_EBroadcastPermissionSubscribers` = `4`
    pub const k_EBroadcastPermissionSubscribers: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBroadcastPermission {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EBroadcastEncoderSetting`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EBroadcastEncoderSetting(pub i32);

impl EBroadcastEncoderSetting {
    /// `k_EBroadcastEncoderBestQuality` = `0`
    pub const k_EBroadcastEncoderBestQuality: Self = Self(0);
    /// `k_EBroadcastEncoderBestPerformance` = `1`
    pub const k_EBroadcastEncoderBestPerformance: Self = Self(1);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EBroadcastEncoderSetting {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECloudGamingPlatform`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECloudGamingPlatform(pub i32);

impl ECloudGamingPlatform {
    /// `k_ECloudGamingPlatformNone` = `0`
    pub const k_ECloudGamingPlatformNone: Self = Self(0);
    /// `k_ECloudGamingPlatformValve` = `1`
    pub const k_ECloudGamingPlatformValve: Self = Self(1);
    /// `k_ECloudGamingPlatformNVIDIA` = `2`
    pub const k_ECloudGamingPlatformNVIDIA: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECloudGamingPlatform {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECompromiseDetectionType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECompromiseDetectionType(pub i32);

impl ECompromiseDetectionType {
    /// `k_ECompromiseDetectionType_None` = `0`
    pub const k_ECompromiseDetectionType_None: Self = Self(0);
    /// `k_ECompromiseDetectionType_TradeEvent` = `1`
    pub const k_ECompromiseDetectionType_TradeEvent: Self = Self(1);
    /// `k_ECompromiseDetectionType_ApiCallRate` = `2`
    pub const k_ECompromiseDetectionType_ApiCallRate: Self = Self(2);
    /// `k_ECompromiseDetectionType_Manual` = `3`
    pub const k_ECompromiseDetectionType_Manual: Self = Self(3);
    /// `k_ECompromiseDetectionType_TicketAction` = `4`
    pub const k_ECompromiseDetectionType_TicketAction: Self = Self(4);
    /// `k_ECompromiseDetectionType_MaliciousRefund` = `5`
    pub const k_ECompromiseDetectionType_MaliciousRefund: Self = Self(5);
    /// `k_ECompromiseDetectionType_Move2FA` = `6`
    pub const k_ECompromiseDetectionType_Move2FA: Self = Self(6);
    /// `k_ECompromiseDetectionType_DeviceType` = `7`
    pub const k_ECompromiseDetectionType_DeviceType: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECompromiseDetectionType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAsyncGameSessionUserState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAsyncGameSessionUserState(pub i32);

impl EAsyncGameSessionUserState {
    /// `k_EAsyncGameSessionUserStateUnknown` = `-1`
    pub const k_EAsyncGameSessionUserStateUnknown: Self = Self(-1);
    /// `k_EAsyncGameSessionUserStateWaitingForOthers` = `0`
    pub const k_EAsyncGameSessionUserStateWaitingForOthers: Self = Self(0);
    /// `k_EAsyncGameSessionUserStateReadyForAction` = `1`
    pub const k_EAsyncGameSessionUserStateReadyForAction: Self = Self(1);
    /// `k_EAsyncGameSessionUserStateDone` = `2`
    pub const k_EAsyncGameSessionUserStateDone: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAsyncGameSessionUserState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAsyncGameSessionUserVisibility`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAsyncGameSessionUserVisibility(pub i32);

impl EAsyncGameSessionUserVisibility {
    /// `k_EAsyncGameSessionUserVisibilityEnvelopeAndSessionList` = `0`
    pub const k_EAsyncGameSessionUserVisibilityEnvelopeAndSessionList: Self = Self(0);
    /// `k_EAsyncGameSessionUserVisibilitySessionListOnly` = `1`
    pub const k_EAsyncGameSessionUserVisibilitySessionListOnly: Self = Self(1);
    /// `k_EAsyncGameSessionUserVisibilityDismissed` = `2`
    pub const k_EAsyncGameSessionUserVisibilityDismissed: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAsyncGameSessionUserVisibility {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGameRecordingType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGameRecordingType(pub i32);

impl EGameRecordingType {
    /// `k_EGameRecordingType_Unknown` = `0`
    pub const k_EGameRecordingType_Unknown: Self = Self(0);
    /// `k_EGameRecordingType_NotRecording` = `1`
    pub const k_EGameRecordingType_NotRecording: Self = Self(1);
    /// `k_EGameRecordingType_ManualRecording` = `2`
    pub const k_EGameRecordingType_ManualRecording: Self = Self(2);
    /// `k_EGameRecordingType_BackgroundRecording` = `3`
    pub const k_EGameRecordingType_BackgroundRecording: Self = Self(3);
    /// `k_EGameRecordingType_Clip` = `4`
    pub const k_EGameRecordingType_Clip: Self = Self(4);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGameRecordingType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EGRMode`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EGRMode(pub i32);

impl EGRMode {
    /// `k_EGRMode_Never` = `0`
    pub const k_EGRMode_Never: Self = Self(0);
    /// `k_EGRMode_Always` = `1`
    pub const k_EGRMode_Always: Self = Self(1);
    /// `k_EGRMode_Manual` = `2`
    pub const k_EGRMode_Manual: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EGRMode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EExportCodec`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EExportCodec(pub i32);

impl EExportCodec {
    /// `k_EExportCodec_Default` = `0`
    pub const k_EExportCodec_Default: Self = Self(0);
    /// `k_EExportCodec_H264` = `1`
    pub const k_EExportCodec_H264: Self = Self(1);
    /// `k_EExportCodec_H265` = `2`
    pub const k_EExportCodec_H265: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EExportCodec {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EProtoAppType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EProtoAppType(pub i32);

impl EProtoAppType {
    /// `k_EAppTypeInvalid` = `0`
    pub const k_EAppTypeInvalid: Self = Self(0);
    /// `k_EAppTypeGame` = `1`
    pub const k_EAppTypeGame: Self = Self(1);
    /// `k_EAppTypeApplication` = `2`
    pub const k_EAppTypeApplication: Self = Self(2);
    /// `k_EAppTypeTool` = `4`
    pub const k_EAppTypeTool: Self = Self(4);
    /// `k_EAppTypeDemo` = `8`
    pub const k_EAppTypeDemo: Self = Self(8);
    /// `k_EAppTypeDeprected` = `16`
    pub const k_EAppTypeDeprected: Self = Self(16);
    /// `k_EAppTypeDLC` = `32`
    pub const k_EAppTypeDLC: Self = Self(32);
    /// `k_EAppTypeGuide` = `64`
    pub const k_EAppTypeGuide: Self = Self(64);
    /// `k_EAppTypeDriver` = `128`
    pub const k_EAppTypeDriver: Self = Self(128);
    /// `k_EAppTypeConfig` = `256`
    pub const k_EAppTypeConfig: Self = Self(256);
    /// `k_EAppTypeHardware` = `512`
    pub const k_EAppTypeHardware: Self = Self(512);
    /// `k_EAppTypeFranchise` = `1024`
    pub const k_EAppTypeFranchise: Self = Self(1024);
    /// `k_EAppTypeVideo` = `2048`
    pub const k_EAppTypeVideo: Self = Self(2048);
    /// `k_EAppTypePlugin` = `4096`
    pub const k_EAppTypePlugin: Self = Self(4096);
    /// `k_EAppTypeMusicAlbum` = `8192`
    pub const k_EAppTypeMusicAlbum: Self = Self(8192);
    /// `k_EAppTypeSeries` = `16384`
    pub const k_EAppTypeSeries: Self = Self(16384);
    /// `k_EAppTypeComic` = `32768`
    pub const k_EAppTypeComic: Self = Self(32768);
    /// `k_EAppTypeBeta` = `65536`
    pub const k_EAppTypeBeta: Self = Self(65536);
    /// `k_EAppTypeShortcut` = `1073741824`
    pub const k_EAppTypeShortcut: Self = Self(1073741824);
    /// `k_EAppTypeDepotOnly` = `-2147483648`
    pub const k_EAppTypeDepotOnly: Self = Self(-2147483648);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EProtoAppType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EAppTestType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EAppTestType(pub i32);

impl EAppTestType {
    /// `k_EAppTestType_BuildReview` = `0`
    pub const k_EAppTestType_BuildReview: Self = Self(0);
    /// `k_EAppTestType_StorePageReview` = `1`
    pub const k_EAppTestType_StorePageReview: Self = Self(1);
    /// `k_EAppTestType_SteamChina_BuildReview` = `2`
    pub const k_EAppTestType_SteamChina_BuildReview: Self = Self(2);
    /// `k_EAppTestType_SteamChina_StorePageReview` = `3`
    pub const k_EAppTestType_SteamChina_StorePageReview: Self = Self(3);
    /// `k_EAppTestType_SteamChinaPlatformOperator_StorePageReview` = `4`
    pub const k_EAppTestType_SteamChinaPlatformOperator_StorePageReview: Self = Self(4);
    /// `k_EAppTestType_SteamChinaPlatformOperator_BuildReview` = `5`
    pub const k_EAppTestType_SteamChinaPlatformOperator_BuildReview: Self = Self(5);
    /// `k_EAppTestType_SteamDeckCompatibilityReview` = `6`
    pub const k_EAppTestType_SteamDeckCompatibilityReview: Self = Self(6);
    /// `k_EAppTestType_SteamFrameCompatibilityReview` = `7`
    pub const k_EAppTestType_SteamFrameCompatibilityReview: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EAppTestType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EChildProcessQueryExitCode`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EChildProcessQueryExitCode(pub i32);

impl EChildProcessQueryExitCode {
    /// `k_EChildProcessQueryExitCode_Success` = `0`
    pub const k_EChildProcessQueryExitCode_Success: Self = Self(0);
    /// `k_EChildProcessQueryExitCode_ErrorCommandline` = `-1`
    pub const k_EChildProcessQueryExitCode_ErrorCommandline: Self = Self(-1);
    /// `k_EChildProcessQueryExitCode_ErrorOther` = `-2`
    pub const k_EChildProcessQueryExitCode_ErrorOther: Self = Self(-2);
    /// `k_EChildProcessQueryExitCode_ErrorUnimplemented` = `-3`
    pub const k_EChildProcessQueryExitCode_ErrorUnimplemented: Self = Self(-3);
    /// `k_EChildProcessQueryExitCode_ErrorFileSave` = `-4`
    pub const k_EChildProcessQueryExitCode_ErrorFileSave: Self = Self(-4);
    /// `k_EChildProcessQueryExitCode_ErrorNotSupportedByPlatform` = `-5`
    pub const k_EChildProcessQueryExitCode_ErrorNotSupportedByPlatform: Self = Self(-5);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EChildProcessQueryExitCode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EWindowsUpdateInstallationImpact`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EWindowsUpdateInstallationImpact(pub i32);

impl EWindowsUpdateInstallationImpact {
    /// `k_EWindowsUpdateInstallationImpact_Unknown` = `-1`
    pub const k_EWindowsUpdateInstallationImpact_Unknown: Self = Self(-1);
    /// `k_EWindowsUpdateInstallationImpact_Normal` = `0`
    pub const k_EWindowsUpdateInstallationImpact_Normal: Self = Self(0);
    /// `k_EWindowsUpdateInstallationImpact_Minor` = `1`
    pub const k_EWindowsUpdateInstallationImpact_Minor: Self = Self(1);
    /// `k_EWindowsUpdateInstallationImpact_ExclusiveHandling` = `2`
    pub const k_EWindowsUpdateInstallationImpact_ExclusiveHandling: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EWindowsUpdateInstallationImpact {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EWindowsUpdateRebootBehavior`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EWindowsUpdateRebootBehavior(pub i32);

impl EWindowsUpdateRebootBehavior {
    /// `k_EWindowsUpdateRebootBehavior_Unknown` = `-1`
    pub const k_EWindowsUpdateRebootBehavior_Unknown: Self = Self(-1);
    /// `k_EWindowsUpdateRebootBehavior_NeverNeedsReboot` = `0`
    pub const k_EWindowsUpdateRebootBehavior_NeverNeedsReboot: Self = Self(0);
    /// `k_EWindowsUpdateRebootBehavior_AlwaysNeedsReboot` = `1`
    pub const k_EWindowsUpdateRebootBehavior_AlwaysNeedsReboot: Self = Self(1);
    /// `k_EWindowsUpdateRebootBehavior_MightNeedReboot` = `2`
    pub const k_EWindowsUpdateRebootBehavior_MightNeedReboot: Self = Self(2);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EWindowsUpdateRebootBehavior {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EExternalSaleEventType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EExternalSaleEventType(pub i32);

impl EExternalSaleEventType {
    /// `k_EExternalSaleEventType_Unknown` = `0`
    pub const k_EExternalSaleEventType_Unknown: Self = Self(0);
    /// `k_EExternalSaleEventType_Publisher` = `1`
    pub const k_EExternalSaleEventType_Publisher: Self = Self(1);
    /// `k_EExternalSaleEventType_Showcase` = `2`
    pub const k_EExternalSaleEventType_Showcase: Self = Self(2);
    /// `k_EExternalSaleEventType_Region` = `3`
    pub const k_EExternalSaleEventType_Region: Self = Self(3);
    /// `k_EExternalSaleEventType_Theme` = `4`
    pub const k_EExternalSaleEventType_Theme: Self = Self(4);
    /// `k_EExternalSaleEventType_Franchise` = `5`
    pub const k_EExternalSaleEventType_Franchise: Self = Self(5);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EExternalSaleEventType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EEnhancedMarketAppearanceStatus`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EEnhancedMarketAppearanceStatus(pub i32);

impl EEnhancedMarketAppearanceStatus {
    /// `k_EnhancedMarketAppearanceStatus_None` = `0`
    pub const k_EnhancedMarketAppearanceStatus_None: Self = Self(0);
    /// `k_EnhancedMarketAppearanceStatus_Pending` = `1`
    pub const k_EnhancedMarketAppearanceStatus_Pending: Self = Self(1);
    /// `k_EnhancedMarketAppearanceStatus_InProgress` = `2`
    pub const k_EnhancedMarketAppearanceStatus_InProgress: Self = Self(2);
    /// `k_EnhancedMarketAppearanceStatus_Completed` = `3`
    pub const k_EnhancedMarketAppearanceStatus_Completed: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EEnhancedMarketAppearanceStatus {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentReportSubjectType`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentReportSubjectType(pub i32);

impl EContentReportSubjectType {
    /// `k_EContentReportSubjectType_Invalid` = `0`
    pub const k_EContentReportSubjectType_Invalid: Self = Self(0);
    /// `k_EContentReportSubjectType_ForumPost` = `1`
    pub const k_EContentReportSubjectType_ForumPost: Self = Self(1);
    /// `k_EContentReportSubjectType_CommentThread` = `2`
    pub const k_EContentReportSubjectType_CommentThread: Self = Self(2);
    /// `k_EContentReportSubjectType_UGCFile` = `3`
    pub const k_EContentReportSubjectType_UGCFile: Self = Self(3);
    /// `k_EContentReportSubjectType_FriendChatMsg` = `4`
    pub const k_EContentReportSubjectType_FriendChatMsg: Self = Self(4);
    /// `k_EContentReportSubjectType_ChatRoomMsg` = `5`
    pub const k_EContentReportSubjectType_ChatRoomMsg: Self = Self(5);
    /// `k_EContentReportSubjectType_ChatGroup` = `6`
    pub const k_EContentReportSubjectType_ChatGroup: Self = Self(6);
    /// `k_EContentReportSubjectType_MAX` = `7`
    pub const k_EContentReportSubjectType_MAX: Self = Self(7);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentReportSubjectType {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EControlledLegalCategoryStatus`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EControlledLegalCategoryStatus(pub i32);

impl EControlledLegalCategoryStatus {
    /// `k_EControlledLegalCategoryStatus_None` = `0`
    pub const k_EControlledLegalCategoryStatus_None: Self = Self(0);
    /// `k_EControlledLegalCategoryStatus_Accused` = `1`
    pub const k_EControlledLegalCategoryStatus_Accused: Self = Self(1);
    /// `k_EControlledLegalCategoryStatus_Convicted` = `2`
    pub const k_EControlledLegalCategoryStatus_Convicted: Self = Self(2);
    /// `k_EControlledLegalCategoryStatus_Acquitted` = `3`
    pub const k_EControlledLegalCategoryStatus_Acquitted: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EControlledLegalCategoryStatus {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentModeratorLevel`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentModeratorLevel(pub i32);

impl EContentModeratorLevel {
    /// `k_EContentModeratorLevel_Any` = `0`
    pub const k_EContentModeratorLevel_Any: Self = Self(0);
    /// `k_EContentModeratorLevel_Supervisor` = `1`
    pub const k_EContentModeratorLevel_Supervisor: Self = Self(1);
    /// `k_EContentModeratorLevel_Valve` = `10`
    pub const k_EContentModeratorLevel_Valve: Self = Self(10);
    /// `k_EContentModeratorLevel_MAX` = `11`
    pub const k_EContentModeratorLevel_MAX: Self = Self(11);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentModeratorLevel {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentReportResolution`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentReportResolution(pub i32);

impl EContentReportResolution {
    /// `k_EContentReportResolution_Unresolved` = `0`
    pub const k_EContentReportResolution_Unresolved: Self = Self(0);
    /// `k_EContentReportResolution_Acquitted` = `1`
    pub const k_EContentReportResolution_Acquitted: Self = Self(1);
    /// `k_EContentReportResolution_Removed` = `2`
    pub const k_EContentReportResolution_Removed: Self = Self(2);
    /// `k_EContentReportResolution_Relabelled` = `3`
    pub const k_EContentReportResolution_Relabelled: Self = Self(3);
    /// `k_EContentReportResolution_Suspicious` = `4`
    pub const k_EContentReportResolution_Suspicious: Self = Self(4);
    /// `k_EContentReportResolution_HarassmentStrike` = `5`
    pub const k_EContentReportResolution_HarassmentStrike: Self = Self(5);
    /// `k_EContentReportResolution_Purged` = `6`
    pub const k_EContentReportResolution_Purged: Self = Self(6);
    /// `k_EContentReportResolution_DisconnectedFromApp` = `7`
    pub const k_EContentReportResolution_DisconnectedFromApp: Self = Self(7);
    /// `k_EContentReportResolution_SuspiciousIncludingUpvoters` = `8`
    pub const k_EContentReportResolution_SuspiciousIncludingUpvoters: Self = Self(8);
    /// `k_EContentReportResolution_VisibilityChanged` = `9`
    pub const k_EContentReportResolution_VisibilityChanged: Self = Self(9);
    /// `k_EContentReportResolution_CountryRestrictionsChanged` = `10`
    pub const k_EContentReportResolution_CountryRestrictionsChanged: Self = Self(10);
    /// `k_EContentReportResolution_RemoveAndWarn` = `11`
    pub const k_EContentReportResolution_RemoveAndWarn: Self = Self(11);
    /// `k_EContentReportResolution_RemoveAndBan` = `12`
    pub const k_EContentReportResolution_RemoveAndBan: Self = Self(12);
    /// `k_EContentReportResolution_RemoveAndKick` = `13`
    pub const k_EContentReportResolution_RemoveAndKick: Self = Self(13);
    /// `k_EContentReportResolution_Sanctioned` = `14`
    pub const k_EContentReportResolution_Sanctioned: Self = Self(14);
    /// `k_EContentReportResolution_Sustained` = `15`
    pub const k_EContentReportResolution_Sustained: Self = Self(15);
    /// `k_EContentReportResolution_Broken` = `16`
    pub const k_EContentReportResolution_Broken: Self = Self(16);
    /// `k_EContentReportResolution_MAX` = `17`
    pub const k_EContentReportResolution_MAX: Self = Self(17);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentReportResolution {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentModerationSanction`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentModerationSanction(pub i32);

impl EContentModerationSanction {
    /// `k_EContentModerationSanction_Invalid` = `0`
    pub const k_EContentModerationSanction_Invalid: Self = Self(0);
    /// `k_EContentModerationSanction_Deleted` = `1`
    pub const k_EContentModerationSanction_Deleted: Self = Self(1);
    /// `k_EContentModerationSanction_CommunityBanned` = `2`
    pub const k_EContentModerationSanction_CommunityBanned: Self = Self(2);
    /// `k_EContentModerationSanction_HubBanned` = `3`
    pub const k_EContentModerationSanction_HubBanned: Self = Self(3);
    /// `k_EContentModerationSanction_TradeBanned` = `4`
    pub const k_EContentModerationSanction_TradeBanned: Self = Self(4);
    /// `k_EContentModerationSanction_CommentHistoryDeleted` = `5`
    pub const k_EContentModerationSanction_CommentHistoryDeleted: Self = Self(5);
    /// `k_EContentModerationSanction_Relabelled` = `6`
    pub const k_EContentModerationSanction_Relabelled: Self = Self(6);
    /// `k_EContentModerationSanction_MarkAsSuspicious` = `7`
    pub const k_EContentModerationSanction_MarkAsSuspicious: Self = Self(7);
    /// `k_EContentModerationSanction_Warned` = `8`
    pub const k_EContentModerationSanction_Warned: Self = Self(8);
    /// `k_EContentModerationSanction_KickedFromGroup` = `9`
    pub const k_EContentModerationSanction_KickedFromGroup: Self = Self(9);
    /// `k_EContentModerationSanction_HarassmentBanned` = `10`
    pub const k_EContentModerationSanction_HarassmentBanned: Self = Self(10);
    /// `k_EContentModerationSanction_HarassmentStrike` = `11`
    pub const k_EContentModerationSanction_HarassmentStrike: Self = Self(11);
    /// `k_EContentModerationSanction_Escalate` = `12`
    pub const k_EContentModerationSanction_Escalate: Self = Self(12);
    /// `k_EContentModerationSanction_MAX` = `13`
    pub const k_EContentModerationSanction_MAX: Self = Self(13);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentModerationSanction {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentReportSubjectAction`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentReportSubjectAction(pub i32);

impl EContentReportSubjectAction {
    /// `k_EContentReportSubjectAction_Invalid` = `0`
    pub const k_EContentReportSubjectAction_Invalid: Self = Self(0);
    /// `k_EContentReportSubjectAction_Unresolved` = `1`
    pub const k_EContentReportSubjectAction_Unresolved: Self = Self(1);
    /// `k_EContentReportSubjectAction_Sanctioned` = `2`
    pub const k_EContentReportSubjectAction_Sanctioned: Self = Self(2);
    /// `k_EContentReportSubjectAction_Acquitted` = `3`
    pub const k_EContentReportSubjectAction_Acquitted: Self = Self(3);
    /// `k_EContentReportSubjectAction_Cancelled` = `4`
    pub const k_EContentReportSubjectAction_Cancelled: Self = Self(4);
    /// `k_EContentReportSubjectAction_Updated` = `5`
    pub const k_EContentReportSubjectAction_Updated: Self = Self(5);
    /// `k_EContentReportSubjectAction_Escalated` = `6`
    pub const k_EContentReportSubjectAction_Escalated: Self = Self(6);
    /// `k_EContentReportSubjectAction_Disputed` = `7`
    pub const k_EContentReportSubjectAction_Disputed: Self = Self(7);
    /// `k_EContentReportSubjectAction_Sustained` = `8`
    pub const k_EContentReportSubjectAction_Sustained: Self = Self(8);
    /// `k_EContentReportSubjectAction_Locked` = `9`
    pub const k_EContentReportSubjectAction_Locked: Self = Self(9);
    /// `k_EContentReportSubjectAction_Unlocked` = `10`
    pub const k_EContentReportSubjectAction_Unlocked: Self = Self(10);
    /// `k_EContentReportSubjectAction_Deleted` = `11`
    pub const k_EContentReportSubjectAction_Deleted: Self = Self(11);
    /// `k_EContentReportSubjectAction_Warned` = `12`
    pub const k_EContentReportSubjectAction_Warned: Self = Self(12);
    /// `k_EContentReportSubjectAction_BannedFromHub` = `13`
    pub const k_EContentReportSubjectAction_BannedFromHub: Self = Self(13);
    /// `k_EContentReportSubjectAction_BannedFromCommunity` = `14`
    pub const k_EContentReportSubjectAction_BannedFromCommunity: Self = Self(14);
    /// `k_EContentReportSubjectAction_TradeBanned` = `15`
    pub const k_EContentReportSubjectAction_TradeBanned: Self = Self(15);
    /// `k_EContentReportSubjectAction_MarkedAsSuspicious` = `16`
    pub const k_EContentReportSubjectAction_MarkedAsSuspicious: Self = Self(16);
    /// `k_EContentReportSubjectAction_ResetContent` = `17`
    pub const k_EContentReportSubjectAction_ResetContent: Self = Self(17);
    /// `k_EContentReportSubjectAction_EscalatedForCSAM` = `18`
    pub const k_EContentReportSubjectAction_EscalatedForCSAM: Self = Self(18);
    /// `k_EContentReportSubjectAction_EscalatedForTerrorism` = `19`
    pub const k_EContentReportSubjectAction_EscalatedForTerrorism: Self = Self(19);
    /// `k_EContentReportSubjectAction_Claimed` = `20`
    pub const k_EContentReportSubjectAction_Claimed: Self = Self(20);
    /// `k_EContentReportSubjectAction_Released` = `21`
    pub const k_EContentReportSubjectAction_Released: Self = Self(21);
    /// `k_EContentReportSubjectAction_PrivateMessaged` = `22`
    pub const k_EContentReportSubjectAction_PrivateMessaged: Self = Self(22);
    /// `k_EContentReportSubjectAction_OwnerDisputed` = `23`
    pub const k_EContentReportSubjectAction_OwnerDisputed: Self = Self(23);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentReportSubjectAction {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EContentReportReason`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentReportReason(pub i32);

impl EContentReportReason {
    /// `k_EContentReportReason_Invalid` = `0`
    pub const k_EContentReportReason_Invalid: Self = Self(0);
    /// `k_EContentReportReason_None` = `1`
    pub const k_EContentReportReason_None: Self = Self(1);
    /// `k_EContentReportReason_Unknown` = `2`
    pub const k_EContentReportReason_Unknown: Self = Self(2);
    /// `k_EContentReportReason_Harassment` = `3`
    pub const k_EContentReportReason_Harassment: Self = Self(3);
    /// `k_EContentReportReason_BullyingAndIntimidation` = `4`
    pub const k_EContentReportReason_BullyingAndIntimidation: Self = Self(4);
    /// `k_EContentReportReason_Stalking` = `5`
    pub const k_EContentReportReason_Stalking: Self = Self(5);
    /// `k_EContentReportReason_Doxxing` = `6`
    pub const k_EContentReportReason_Doxxing: Self = Self(6);
    /// `k_EContentReportReason_OtherHarassment` = `7`
    pub const k_EContentReportReason_OtherHarassment: Self = Self(7);
    /// `k_EContentReportReason_EncouragingViolence` = `8`
    pub const k_EContentReportReason_EncouragingViolence: Self = Self(8);
    /// `k_EContentReportReason_EncouragingSelfHarm` = `9`
    pub const k_EContentReportReason_EncouragingSelfHarm: Self = Self(9);
    /// `k_EContentReportReason_EncouragingSuicide` = `10`
    pub const k_EContentReportReason_EncouragingSuicide: Self = Self(10);
    /// `k_EContentReportReason_OtherViolenceOrSelfHarm` = `11`
    pub const k_EContentReportReason_OtherViolenceOrSelfHarm: Self = Self(11);
    /// `k_EContentReportReason_PhishingOrAccountTheft` = `12`
    pub const k_EContentReportReason_PhishingOrAccountTheft: Self = Self(12);
    /// `k_EContentReportReason_AttemptedScamming` = `13`
    pub const k_EContentReportReason_AttemptedScamming: Self = Self(13);
    /// `k_EContentReportReason_LinkingToMaliciousContent` = `14`
    pub const k_EContentReportReason_LinkingToMaliciousContent: Self = Self(14);
    /// `k_EContentReportReason_Impersonation` = `15`
    pub const k_EContentReportReason_Impersonation: Self = Self(15);
    /// `k_EContentReportReason_OtherScamsAndTheft` = `16`
    pub const k_EContentReportReason_OtherScamsAndTheft: Self = Self(16);
    /// `k_EContentReportReason_EncouragingTerrorism` = `17`
    pub const k_EContentReportReason_EncouragingTerrorism: Self = Self(17);
    /// `k_EContentReportReason_OrganizingTerrorism` = `18`
    pub const k_EContentReportReason_OrganizingTerrorism: Self = Self(18);
    /// `k_EContentReportReason_OtherTerrorism` = `19`
    pub const k_EContentReportReason_OtherTerrorism: Self = Self(19);
    /// `k_EContentReportReason_TargetedAbuse` = `20`
    pub const k_EContentReportReason_TargetedAbuse: Self = Self(20);
    /// `k_EContentReportReason_NamingAndShaming` = `21`
    pub const k_EContentReportReason_NamingAndShaming: Self = Self(21);
    /// `k_EContentReportReason_Discrimination` = `22`
    pub const k_EContentReportReason_Discrimination: Self = Self(22);
    /// `k_EContentReportReason_OtherAbuse` = `23`
    pub const k_EContentReportReason_OtherAbuse: Self = Self(23);
    /// `k_EContentReportReason_Trolling` = `24`
    pub const k_EContentReportReason_Trolling: Self = Self(24);
    /// `k_EContentReportReason_Baiting` = `25`
    pub const k_EContentReportReason_Baiting: Self = Self(25);
    /// `k_EContentReportReason_Derailing` = `26`
    pub const k_EContentReportReason_Derailing: Self = Self(26);
    /// `k_EContentReportReason_OtherDisruptive` = `27`
    pub const k_EContentReportReason_OtherDisruptive: Self = Self(27);
    /// `k_EContentReportReason_Spam` = `28`
    pub const k_EContentReportReason_Spam: Self = Self(28);
    /// `k_EContentReportReason_Begging` = `29`
    pub const k_EContentReportReason_Begging: Self = Self(29);
    /// `k_EContentReportReason_Reposting` = `30`
    pub const k_EContentReportReason_Reposting: Self = Self(30);
    /// `k_EContentReportReason_OtherOffTopic` = `31`
    pub const k_EContentReportReason_OtherOffTopic: Self = Self(31);
    /// `k_EContentReportReason_CSAMSexualContent` = `32`
    pub const k_EContentReportReason_CSAMSexualContent: Self = Self(32);
    /// `k_EContentReportReason_CSAMGroomingOrEnticement` = `33`
    pub const k_EContentReportReason_CSAMGroomingOrEnticement: Self = Self(33);
    /// `k_EContentReportReason_CSAMOther` = `34`
    pub const k_EContentReportReason_CSAMOther: Self = Self(34);
    /// `k_EContentReportReason_NudityOrSexualContent` = `35`
    pub const k_EContentReportReason_NudityOrSexualContent: Self = Self(35);
    /// `k_EContentReportReason_NonConsensualMaterial` = `36`
    pub const k_EContentReportReason_NonConsensualMaterial: Self = Self(36);
    /// `k_EContentReportReason_Advertising` = `37`
    pub const k_EContentReportReason_Advertising: Self = Self(37);
    /// `k_EContentReportReason_ReferralLinks` = `38`
    pub const k_EContentReportReason_ReferralLinks: Self = Self(38);
    /// `k_EContentReportReason_Gambling` = `39`
    pub const k_EContentReportReason_Gambling: Self = Self(39);
    /// `k_EContentReportReason_Raffles` = `40`
    pub const k_EContentReportReason_Raffles: Self = Self(40);
    /// `k_EContentReportReason_OtherCommercialActivity` = `41`
    pub const k_EContentReportReason_OtherCommercialActivity: Self = Self(41);
    /// `k_EContentReportReason_InauthenticReview` = `42`
    pub const k_EContentReportReason_InauthenticReview: Self = Self(42);
    /// `k_EContentReportReason_HiddenAdvertisementOrCommercialCommunication` = `43`
    pub const k_EContentReportReason_HiddenAdvertisementOrCommercialCommunication: Self = Self(43);
    /// `k_EContentReportReason_MisleadingInformationAboutGoodsOrServices` = `44`
    pub const k_EContentReportReason_MisleadingInformationAboutGoodsOrServices: Self = Self(44);
    /// `k_EContentReportReason_MisleadingInformationAboutConsumerRights` = `45`
    pub const k_EContentReportReason_MisleadingInformationAboutConsumerRights: Self = Self(45);
    /// `k_EContentReportReason_NoncomplianceWithPricingRegulations` = `46`
    pub const k_EContentReportReason_NoncomplianceWithPricingRegulations: Self = Self(46);
    /// `k_EContentReportReason_RightToBeForgottenViolation` = `47`
    pub const k_EContentReportReason_RightToBeForgottenViolation: Self = Self(47);
    /// `k_EContentReportReason_MissingProcessingGroundForData` = `48`
    pub const k_EContentReportReason_MissingProcessingGroundForData: Self = Self(48);
    /// `k_EContentReportReason_OtherDataProtectionAndPrivacyViolation` = `49`
    pub const k_EContentReportReason_OtherDataProtectionAndPrivacyViolation: Self = Self(49);
    /// `k_EContentReportReason_GenderedHarassment` = `50`
    pub const k_EContentReportReason_GenderedHarassment: Self = Self(50);
    /// `k_EContentReportReason_GenderedBullyingAndIntimidation` = `51`
    pub const k_EContentReportReason_GenderedBullyingAndIntimidation: Self = Self(51);
    /// `k_EContentReportReason_GenderedStalking` = `52`
    pub const k_EContentReportReason_GenderedStalking: Self = Self(52);
    /// `k_EContentReportReason_GenderedDoxxing` = `53`
    pub const k_EContentReportReason_GenderedDoxxing: Self = Self(53);
    /// `k_EContentReportReason_GenderedOtherHarassment` = `54`
    pub const k_EContentReportReason_GenderedOtherHarassment: Self = Self(54);
    /// `k_EContentReportReason_GenderedEncouragingViolence` = `55`
    pub const k_EContentReportReason_GenderedEncouragingViolence: Self = Self(55);
    /// `k_EContentReportReason_GenderedTargetedAbuse` = `56`
    pub const k_EContentReportReason_GenderedTargetedAbuse: Self = Self(56);
    /// `k_EContentReportReason_CSAMFakedSexualContent` = `57`
    pub const k_EContentReportReason_CSAMFakedSexualContent: Self = Self(57);
    /// `k_EContentReportReason_GenderedNonConsensualMaterial` = `58`
    pub const k_EContentReportReason_GenderedNonConsensualMaterial: Self = Self(58);
    /// `k_EContentReportReason_FakedGenderedNonConsensualMaterial` = `59`
    pub const k_EContentReportReason_FakedGenderedNonConsensualMaterial: Self = Self(59);
    /// `k_EContentReportReason_FakedNonConsensualMaterial` = `60`
    pub const k_EContentReportReason_FakedNonConsensualMaterial: Self = Self(60);
    /// `k_EContentReportReason_NegativeEffectonDiscourseOrElections` = `61`
    pub const k_EContentReportReason_NegativeEffectonDiscourseOrElections: Self = Self(61);
    /// `k_EContentReportReason_QuotesModeratedContent` = `62`
    pub const k_EContentReportReason_QuotesModeratedContent: Self = Self(62);
    /// `k_EContentReportReason_CredibleThreatOfViolence` = `63`
    pub const k_EContentReportReason_CredibleThreatOfViolence: Self = Self(63);
    /// `k_EContentReportReason_AutoCreatedOnModeration` = `64`
    pub const k_EContentReportReason_AutoCreatedOnModeration: Self = Self(64);
    /// `k_EContentReportReason_Piracy` = `65`
    pub const k_EContentReportReason_Piracy: Self = Self(65);
    /// `k_EContentReportReason_ToSViolation` = `66`
    pub const k_EContentReportReason_ToSViolation: Self = Self(66);
    /// `k_EContentReportReason_Miscategorized` = `67`
    pub const k_EContentReportReason_Miscategorized: Self = Self(67);
    /// `k_EContentReportReason_BypassingProfanityFilter` = `68`
    pub const k_EContentReportReason_BypassingProfanityFilter: Self = Self(68);
    /// `k_EContentReportReason_MAX` = `69`
    pub const k_EContentReportReason_MAX: Self = Self(69);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentReportReason {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EResolutionAutomation`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EResolutionAutomation(pub i32);

impl EResolutionAutomation {
    /// `k_EResolutionAutomation_Manual` = `0`
    pub const k_EResolutionAutomation_Manual: Self = Self(0);
    /// `k_EResolutionAutomation_PartiallyAutomated` = `1`
    pub const k_EResolutionAutomation_PartiallyAutomated: Self = Self(1);
    /// `k_EResolutionAutomation_FullyAutomated` = `2`
    pub const k_EResolutionAutomation_FullyAutomated: Self = Self(2);
    /// `k_EResolutionAutomation_MAX` = `3`
    pub const k_EResolutionAutomation_MAX: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EResolutionAutomation {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPressOutletAction`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPressOutletAction(pub i32);

impl EPressOutletAction {
    /// `k_EPressOutletAction_Invalid` = `0`
    pub const k_EPressOutletAction_Invalid: Self = Self(0);
    /// `k_EPressOutletAction_Granted` = `1`
    pub const k_EPressOutletAction_Granted: Self = Self(1);
    /// `k_EPressOutletAction_Removed` = `2`
    pub const k_EPressOutletAction_Removed: Self = Self(2);
    /// `k_EPressOutletAction_Created` = `3`
    pub const k_EPressOutletAction_Created: Self = Self(3);
    /// `k_EPressOutletAction_Updated` = `4`
    pub const k_EPressOutletAction_Updated: Self = Self(4);
    /// `k_EPressOutletAction_Deleted` = `5`
    pub const k_EPressOutletAction_Deleted: Self = Self(5);
    /// `k_EPressOutletAction_Undeleted` = `6`
    pub const k_EPressOutletAction_Undeleted: Self = Self(6);
    /// `k_EPressOutletAction_StagedAdd` = `7`
    pub const k_EPressOutletAction_StagedAdd: Self = Self(7);
    /// `k_EPressOutletAction_StagedDelete` = `8`
    pub const k_EPressOutletAction_StagedDelete: Self = Self(8);
    /// `k_EPressOutletAction_EnterStaging` = `9`
    pub const k_EPressOutletAction_EnterStaging: Self = Self(9);
    /// `k_EPressOutletAction_ExitStaging` = `10`
    pub const k_EPressOutletAction_ExitStaging: Self = Self(10);
    /// `k_EPressOutletAction_ReverseStagedAdd` = `11`
    pub const k_EPressOutletAction_ReverseStagedAdd: Self = Self(11);
    /// `k_EPressOutletAction_ReverseStagedDelete` = `12`
    pub const k_EPressOutletAction_ReverseStagedDelete: Self = Self(12);
    /// `k_EPressOutletAction_MAX` = `13`
    pub const k_EPressOutletAction_MAX: Self = Self(13);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPressOutletAction {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `EPressOutletMemberPendingState`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EPressOutletMemberPendingState(pub i32);

impl EPressOutletMemberPendingState {
    /// `k_EPressOutletMemberPendingState_Member` = `0`
    pub const k_EPressOutletMemberPendingState_Member: Self = Self(0);
    /// `k_EPressOutletMemberPendingState_StagedDelete` = `1`
    pub const k_EPressOutletMemberPendingState_StagedDelete: Self = Self(1);
    /// `k_EPressOutletMemberPendingState_StagedAdd` = `2`
    pub const k_EPressOutletMemberPendingState_StagedAdd: Self = Self(2);
    /// `k_EPressOutletMemberPendingState_MAX` = `3`
    pub const k_EPressOutletMemberPendingState_MAX: Self = Self(3);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EPressOutletMemberPendingState {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// `ECommentDeleteReason`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct ECommentDeleteReason(pub i32);

impl ECommentDeleteReason {
    /// `k_ECommentDeleteReason_Invalid` = `0`
    pub const k_ECommentDeleteReason_Invalid: Self = Self(0);
    /// `k_ECommentDeleteReason_User` = `1`
    pub const k_ECommentDeleteReason_User: Self = Self(1);
    /// `k_ECommentDeleteReason_ThreadOwner` = `2`
    pub const k_ECommentDeleteReason_ThreadOwner: Self = Self(2);
    /// `k_ECommentDeleteReason_Moderator` = `3`
    pub const k_ECommentDeleteReason_Moderator: Self = Self(3);
    /// `k_ECommentDeleteReason_Support` = `4`
    pub const k_ECommentDeleteReason_Support: Self = Self(4);
    /// `k_ECommentDeleteReason_Spam` = `5`
    pub const k_ECommentDeleteReason_Spam: Self = Self(5);
    /// `k_ECommentDeleteReason_AccountDeletion` = `6`
    pub const k_ECommentDeleteReason_AccountDeletion: Self = Self(6);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for ECommentDeleteReason {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
