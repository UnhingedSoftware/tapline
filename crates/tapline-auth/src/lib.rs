//! Signing in to Steam with an account.

mod local;
mod password;
mod store;

pub use local::{
    LocalAccount, discover, discover_in, libraries, most_recent, parse_libraries, parse_login_users,
};
pub use password::{PasswordError, PublicKey, encrypt_password};
pub use store::{StoredToken, TokenStore, TokenStoreError};

use std::fmt;

/// How Steam wants a login confirmed, from `EAuthSessionGuardType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardType {
    /// Nothing further needed.
    None,
    /// A code emailed to the account.
    EmailCode,
    /// A code from the mobile authenticator.
    DeviceCode,
    /// Approve it in the Steam mobile app.
    DeviceConfirmation,
    /// Confirm by email link.
    EmailConfirmation,
    /// A machine token established by an earlier login.
    MachineToken,
    /// A type this build does not know about, kept with its number.
    Unknown(i32),
}

impl GuardType {
    /// The wire value; Steam refuses a code of a different kind than it asked.
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

    /// Decodes the wire value.
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

    /// Whether the user must supply a code, as opposed to approving elsewhere.
    #[must_use]
    pub const fn needs_a_code(self) -> bool {
        matches!(self, Self::EmailCode | Self::DeviceCode)
    }

    /// A description for a prompt.
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

/// `EAuthTokenPlatformType::SteamClient`; only this platform's tokens can fetch depot keys.
pub const PLATFORM_STEAM_CLIENT: i32 = 1;

/// The name a login shows in the account's device list.
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
