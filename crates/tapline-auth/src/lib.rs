mod local;
mod password;
mod store;

pub use local::{
    LocalAccount, discover, discover_in, libraries, most_recent, parse_libraries, parse_login_users,
};
pub use password::{PasswordError, PublicKey, encrypt_password};
pub use store::{StoredToken, TokenStore, TokenStoreError};

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardType {
    None,
    EmailCode,
    DeviceCode,
    DeviceConfirmation,
    EmailConfirmation,
    MachineToken,
    Unknown(i32),
}

impl GuardType {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::None => 0,
            Self::EmailCode => 2,
            Self::DeviceCode => 3,
            Self::DeviceConfirmation => 4,
            Self::EmailConfirmation => 5,
            Self::MachineToken => 6,
            Self::Unknown(value) => value,
        }
    }

    #[must_use]
    pub const fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::None,
            2 => Self::EmailCode,
            3 => Self::DeviceCode,
            4 => Self::DeviceConfirmation,
            5 => Self::EmailConfirmation,
            6 => Self::MachineToken,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn needs_a_code(self) -> bool {
        matches!(self, Self::EmailCode | Self::DeviceCode)
    }

    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::None => "no confirmation needed",
            Self::EmailCode => "a Steam Guard code sent by email",
            Self::DeviceCode => "a Steam Guard code from the mobile authenticator",
            Self::DeviceConfirmation => "approval in the Steam mobile app",
            Self::EmailConfirmation => "confirmation by email link",
            Self::MachineToken => "a machine token from a previous login",
            Self::Unknown(_) => "an unrecognised confirmation type",
        }
    }
}

impl fmt::Display for GuardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(value) => write!(f, "unsupported confirmation type {value}"),
            other => f.write_str(other.describe()),
        }
    }
}

pub const PLATFORM_STEAM_CLIENT: i32 = 1;

pub const DEVICE_NAME: &str = "tapline";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_types_decode_the_way_steam_numbers_them() {
        assert_eq!(GuardType::from_i32(0), GuardType::None);
        assert_eq!(GuardType::from_i32(2), GuardType::EmailCode);
        assert_eq!(GuardType::from_i32(3), GuardType::DeviceCode);
        assert_eq!(GuardType::from_i32(4), GuardType::DeviceConfirmation);
    }

    #[test]
    fn an_unknown_guard_type_keeps_its_number() {
        let guard = GuardType::from_i32(9);
        assert_eq!(guard, GuardType::Unknown(9));
        assert!(guard.to_string().contains('9'));
    }

    #[test]
    fn code_prompts_are_distinguished_from_approvals() {
        assert!(GuardType::EmailCode.needs_a_code());
        assert!(GuardType::DeviceCode.needs_a_code());
        assert!(!GuardType::DeviceConfirmation.needs_a_code());
        assert!(!GuardType::None.needs_a_code());
    }
}
