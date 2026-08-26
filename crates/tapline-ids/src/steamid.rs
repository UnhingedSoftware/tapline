//! The 64-bit SteamID and its bit layout.
//!
//! ```text
//!  63        56 55      52 51                 32 31                        0
//! +------------+----------+---------------------+---------------------------+
//! |  universe  |   type   |      instance       |        account id         |
//! +------------+----------+---------------------+---------------------------+
//!      8 bits     4 bits         20 bits                  32 bits
//! ```
//!
//! Anonymous logons — the common case for dedicated servers — come back with an
//! [`AccountType::AnonUser`] id whose account number Steam picked, so parsing
//! this correctly is not optional even for a workload that never signs in.

use std::fmt;

/// Which Steam universe an id belongs to. Public is the only one reachable from
/// outside Valve, but the field still has to round-trip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Universe {
    /// Unset or invalid.
    Invalid,
    /// The live, public Steam network.
    Public,
    /// Valve-internal.
    Beta,
    /// Valve-internal.
    Internal,
    /// Valve-internal.
    Dev,
    /// A universe this build does not know about.
    Unknown(u8),
}

impl Universe {
    /// The wire encoding.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Invalid => 0,
            Self::Public => 1,
            Self::Beta => 2,
            Self::Internal => 3,
            Self::Dev => 4,
            Self::Unknown(v) => v,
        }
    }

    /// Decodes the wire encoding, preserving anything unrecognised rather than
    /// collapsing it to invalid.
    #[must_use]
    pub const fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Invalid,
            1 => Self::Public,
            2 => Self::Beta,
            3 => Self::Internal,
            4 => Self::Dev,
            other => Self::Unknown(other),
        }
    }
}

/// What kind of account an id names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// Unset or invalid.
    Invalid,
    /// A person.
    Individual,
    /// A multiseat (cybercafe) account.
    Multiseat,
    /// A persistent game server.
    GameServer,
    /// An anonymous game server.
    AnonGameServer,
    /// A pending account.
    Pending,
    /// A content server.
    ContentServer,
    /// A group.
    Clan,
    /// A chat room.
    Chat,
    /// A peer-to-peer superseeder.
    ConsoleUser,
    /// An anonymous user — what an anonymous logon becomes.
    AnonUser,
    /// A type this build does not know about.
    Unknown(u8),
}

impl AccountType {
    /// The wire encoding.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Invalid => 0,
            Self::Individual => 1,
            Self::Multiseat => 2,
            Self::GameServer => 3,
            Self::AnonGameServer => 4,
            Self::Pending => 5,
            Self::ContentServer => 6,
            Self::Clan => 7,
            Self::Chat => 8,
            Self::ConsoleUser => 9,
            Self::AnonUser => 10,
            Self::Unknown(v) => v,
        }
    }

    /// Decodes the wire encoding.
    #[must_use]
    pub const fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Invalid,
            1 => Self::Individual,
            2 => Self::Multiseat,
            3 => Self::GameServer,
            4 => Self::AnonGameServer,
            5 => Self::Pending,
            6 => Self::ContentServer,
            7 => Self::Clan,
            8 => Self::Chat,
            9 => Self::ConsoleUser,
            10 => Self::AnonUser,
            other => Self::Unknown(other),
        }
    }

    /// The single letter Steam uses for this type in the `[U:1:1234]` rendering.
    #[must_use]
    pub const fn letter(self) -> char {
        match self {
            Self::Individual => 'U',
            Self::Multiseat => 'M',
            Self::GameServer => 'G',
            Self::AnonGameServer => 'A',
            Self::Pending => 'P',
            Self::ContentServer => 'C',
            Self::Clan => 'g',
            Self::Chat => 'T',
            Self::ConsoleUser | Self::AnonUser => 'a',
            Self::Invalid | Self::Unknown(_) => 'I',
        }
    }
}

/// A 64-bit Steam identifier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct SteamId(pub u64);

/// The instance value a normal desktop client uses.
const INSTANCE_DESKTOP: u32 = 1;

impl SteamId {
    /// Wraps a raw 64-bit id.
    #[inline]
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Builds an id from its parts. Values wider than their field are masked to
    /// fit rather than silently corrupting a neighbouring field.
    #[must_use]
    pub const fn from_parts(
        universe: Universe,
        account_type: AccountType,
        instance: u32,
        account_id: u32,
    ) -> Self {
        let raw = ((universe.to_u8() as u64) << 56)
            | (((account_type.to_u8() as u64) & 0xF) << 52)
            | (((instance as u64) & 0xF_FFFF) << 32)
            | (account_id as u64);
        Self(raw)
    }

    /// The raw 64-bit value, as it goes on the wire.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// The low 32 bits: the account number.
    #[inline]
    #[must_use]
    pub const fn account_id(self) -> u32 {
        self.0 as u32
    }

    /// The 20-bit instance field.
    #[inline]
    #[must_use]
    pub const fn instance(self) -> u32 {
        ((self.0 >> 32) & 0xF_FFFF) as u32
    }

    /// The 4-bit account type field.
    #[inline]
    #[must_use]
    pub const fn account_type(self) -> AccountType {
        AccountType::from_u8(((self.0 >> 52) & 0xF) as u8)
    }

    /// The 8-bit universe field.
    #[inline]
    #[must_use]
    pub const fn universe(self) -> Universe {
        Universe::from_u8((self.0 >> 56) as u8)
    }

    /// Whether this id names something real.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        !matches!(self.account_type(), AccountType::Invalid)
            && !matches!(self.universe(), Universe::Invalid)
    }

    /// The id an anonymous logon starts from: no account number yet, Steam fills
    /// one in and hands it back in the logon response.
    #[must_use]
    pub const fn anonymous() -> Self {
        Self::from_parts(Universe::Public, AccountType::AnonUser, INSTANCE_DESKTOP, 0)
    }

    /// Renders the modern `[U:1:1234]` form.
    #[must_use]
    pub fn to_steam3(self) -> String {
        format!(
            "[{}:{}:{}]",
            self.account_type().letter(),
            self.universe().to_u8(),
            self.account_id()
        )
    }
}

impl fmt::Debug for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SteamId({} {})", self.0, self.to_steam3())
    }
}

impl fmt::Display for SteamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<u64> for SteamId {
    #[inline]
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<SteamId> for u64 {
    #[inline]
    fn from(v: SteamId) -> Self {
        v.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_individual_id_decodes_field_by_field() {
        // A real public individual id: account 22202, desktop instance.
        let id = SteamId::new(76_561_197_960_287_930);
        assert_eq!(id.universe(), Universe::Public);
        assert_eq!(id.account_type(), AccountType::Individual);
        assert_eq!(id.instance(), INSTANCE_DESKTOP);
        assert_eq!(id.account_id(), 22_202);
        assert_eq!(id.to_steam3(), "[U:1:22202]");
        assert!(id.is_valid());
    }

    #[test]
    fn parts_round_trip_through_the_packed_form() {
        let id = SteamId::from_parts(Universe::Public, AccountType::Individual, 1, 22_202);
        assert_eq!(id.raw(), 76_561_197_960_287_930);
    }

    #[test]
    fn oversized_fields_are_masked_not_smeared_into_neighbours() {
        // An instance wider than 20 bits must not corrupt the type field above
        // it; getting this wrong turns an individual account into a chat room.
        let id = SteamId::from_parts(Universe::Public, AccountType::Individual, u32::MAX, 7);
        assert_eq!(id.account_type(), AccountType::Individual);
        assert_eq!(id.instance(), 0xF_FFFF);
        assert_eq!(id.account_id(), 7);
    }

    #[test]
    fn anonymous_id_is_the_shape_steam_expects_for_an_anon_logon() {
        let id = SteamId::anonymous();
        assert_eq!(id.account_type(), AccountType::AnonUser);
        assert_eq!(id.universe(), Universe::Public);
        assert_eq!(id.account_id(), 0);
    }

    #[test]
    fn unknown_enum_values_survive_a_round_trip() {
        // Valve adds types; a build that collapses them to Invalid would send
        // back a different id than it received.
        assert_eq!(AccountType::from_u8(13).to_u8(), 13);
        assert_eq!(Universe::from_u8(200).to_u8(), 200);
    }
}
