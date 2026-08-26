//! Signing in to Steam with an account.
//!
//! Anonymous logon covers every dedicated server, and it is what tapline uses by
//! default. This is for the rest: an app that is owned rather than
//! anonymously accessible needs an account behind it.
//!
//! # What this does with a password
//!
//! As little as possible, for as short a time as possible.
//!
//! * The password is encrypted with a **per-account RSA key fetched from Steam**
//!   for this login. There is no long-lived Valve key involved, and nothing to
//!   hardcode.
//! * It is **never written to disk**, never logged, and zeroed as soon as it has
//!   been encrypted.
//! * What gets persisted, and only when the caller asks, is the **refresh
//!   token** — which Steam can revoke and which is worth less than the password
//!   it replaces.
//!
//! # QR is the better default
//!
//! [`QrSession`] never sees a password at all: Steam issues a challenge URL, the
//! user approves it in the mobile app, and the session polls until a token comes
//! back. For an interactive login it is both easier and safer, and it is the
//! flow that can be tested end to end without anyone typing a secret.

mod password;
mod store;

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
    /// Something this build does not know about.
    ///
    /// Kept with its number: Valve adds these, and "unsupported confirmation
    /// type 9" is a message someone can act on.
    Unknown(i32),
}

impl GuardType {
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

    /// Whether the user must supply a code for this.
    ///
    /// Distinguishes "type a number" from "tap approve on your phone", which is
    /// the difference between prompting and waiting.
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

/// `EAuthTokenPlatformType`: what kind of client is signing in.
///
/// Steam issues tokens with different scopes per platform, and a
/// `SteamClient` token is the one that can fetch depot keys — which is the
/// entire reason to log in at all here.
pub const PLATFORM_STEAM_CLIENT: i32 = 1;

/// The name a login shows in the account's device list.
///
/// Honest about what it is: someone reviewing their authorised devices should
/// be able to tell what added one.
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
        // "unsupported confirmation type 9" is actionable; "unknown error" is
        // not.
        let guard = GuardType::from_i32(9);
        assert_eq!(guard, GuardType::Unknown(9));
        assert!(guard.to_string().contains('9'));
    }

    #[test]
    fn code_prompts_are_distinguished_from_approvals() {
        // The difference between asking the user to type something and telling
        // them to look at their phone.
        assert!(GuardType::EmailCode.needs_a_code());
        assert!(GuardType::DeviceCode.needs_a_code());
        assert!(!GuardType::DeviceConfirmation.needs_a_code());
        assert!(!GuardType::None.needs_a_code());
    }
}
