use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Universe {
    Invalid,
    Public,
    Beta,
    Internal,
    Dev,
    Unknown(u8),
}

impl Universe {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountType {
    Invalid,
    Individual,
    Multiseat,
    GameServer,
    AnonGameServer,
    Pending,
    ContentServer,
    Clan,
    Chat,
    ConsoleUser,
    AnonUser,
    Unknown(u8),
}

impl AccountType {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct SteamId(pub u64);

const INSTANCE_DESKTOP: u32 = 1;

impl SteamId {
    #[inline]
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

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

    #[inline]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn account_id(self) -> u32 {
        self.0 as u32
    }

    #[inline]
    #[must_use]
    pub const fn instance(self) -> u32 {
        ((self.0 >> 32) & 0xF_FFFF) as u32
    }

    #[inline]
    #[must_use]
    pub const fn account_type(self) -> AccountType {
        AccountType::from_u8(((self.0 >> 52) & 0xF) as u8)
    }

    #[inline]
    #[must_use]
    pub const fn universe(self) -> Universe {
        Universe::from_u8((self.0 >> 56) as u8)
    }

    #[must_use]
    pub const fn is_valid(self) -> bool {
        !matches!(self.account_type(), AccountType::Invalid)
            && !matches!(self.universe(), Universe::Invalid)
    }

    #[must_use]
    pub const fn anonymous() -> Self {
        Self::from_parts(Universe::Public, AccountType::AnonUser, INSTANCE_DESKTOP, 0)
    }

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
        assert_eq!(AccountType::from_u8(13).to_u8(), 13);
        assert_eq!(Universe::from_u8(200).to_u8(), 200);
    }
}
