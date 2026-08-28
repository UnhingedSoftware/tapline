//! Signing in with an account.
//!
//! Two flows, and the QR one is better for anything interactive:
//!
//! * [`begin_qr`] asks Steam for a challenge URL. The user approves it in the
//!   mobile app, [`poll`] returns a token, and **no password is ever typed,
//!   transmitted or held in memory**.
//! * [`begin_password`] is the traditional flow, for a script that has
//!   credentials in a secret store and no human present.
//!
//! Both end in the same place: a refresh token, which is what
//! `Session::logon_with_token` uses and what the token store persists.
//!
//! # Nothing here is needed for a dedicated server
//!
//! Anonymous logon covers every dedicated-server depot, which is what this
//! project mostly exists for. This module is for the case where an owned app has
//! to be installed, and none of it runs otherwise.

use crate::InstallError;
use std::fmt;
use tapline_auth::{DEVICE_NAME, GuardType, PLATFORM_STEAM_CLIENT, PublicKey};

/// A login in progress.
#[derive(Debug, Clone)]
pub struct PendingLogin {
    /// Steam's identifier for this attempt.
    pub client_id: u64,
    /// The opaque request id to poll with.
    pub request_id: Vec<u8>,
    /// How often Steam wants to be polled, in seconds.
    ///
    /// Honoured rather than ignored: polling faster is how a login gets rate
    /// limited, and Steam is explicit about the interval it wants.
    pub interval: f32,
    /// How the user must confirm, in the order Steam listed them.
    pub confirmations: Vec<GuardType>,
    /// For a QR login, the URL to render as a code.
    pub challenge_url: Option<String>,
    /// The account name, once Steam knows it.
    pub account: Option<String>,
    /// The SteamID this attempt is for, which a Guard code submission needs.
    pub steam_id: u64,
}

impl PendingLogin {
    /// Whether typing a code is *among* the ways to confirm this login.
    ///
    /// Not the same as "the user must type a code". A real QR login comes back
    /// offering `[DeviceConfirmation, DeviceCode]` — approve it on the phone
    /// **or** type the authenticator code — so the list is a set of
    /// alternatives, not a set of requirements. Reading it as requirements is
    /// what an earlier version of this did, and it would have prompted for a
    /// code during a QR login that needed no such thing.
    #[must_use]
    pub fn accepts_a_code(&self) -> bool {
        self.confirmations.iter().any(|guard| guard.needs_a_code())
    }

    /// Whether a code is the *only* way to confirm.
    ///
    /// This is the question a caller deciding whether to prompt should ask.
    #[must_use]
    pub fn requires_a_code(&self) -> bool {
        self.challenge_url.is_none()
            && !self.confirmations.is_empty()
            && self.confirmations.iter().all(|guard| guard.needs_a_code())
    }

    /// What to tell a person waiting to log in.
    ///
    /// The QR URL wins when there is one, because scanning is the path that
    /// needs no typing — but any code alternative is mentioned, since someone
    /// without the phone to hand needs to know it exists.
    #[must_use]
    pub fn instruction(&self) -> String {
        match &self.challenge_url {
            Some(url) => {
                let mut line = format!("Scan this in the Steam mobile app: {url}");
                if self.accepts_a_code() {
                    line.push_str(" (or confirm with a Steam Guard code)");
                }
                line
            }
            None => match self.confirmations.first() {
                Some(guard) => format!("Steam needs {guard}"),
                None => "Waiting for Steam".to_owned(),
            },
        }
    }
}

/// What a poll found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollOutcome {
    /// Still waiting for the user.
    Pending {
        /// Whether the user has interacted at all yet, which is worth showing:
        /// it distinguishes "they have not looked at their phone" from
        /// "they are part-way through".
        had_interaction: bool,
    },
    /// Steam moved the session; poll with the new identifiers.
    ///
    /// Happens on a QR login when the code is refreshed. Ignoring it means
    /// polling a dead session forever.
    Moved {
        /// The new client id.
        client_id: u64,
        /// The new URL to display.
        challenge_url: Option<String>,
    },
    /// Done.
    Complete {
        /// The account that signed in.
        account: String,
        /// The refresh token, which is what to persist.
        refresh_token: String,
        /// The access token for this session.
        access_token: String,
    },
}

/// What went wrong signing in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginError {
    /// Steam refused, with its own result code.
    ///
    /// Kept as a number because the difference between "wrong password",
    /// "rate limited" and "needs a Guard code" decides what the caller does
    /// next, and collapsing them to "login failed" throws that away.
    Refused {
        /// Steam's `EResult`.
        eresult: i32,
        /// Any message Steam attached.
        message: Option<String>,
    },
    /// The RSA key or the password could not be handled.
    Password(String),
    /// Steam did not return what the flow needs to continue.
    Incomplete(&'static str),
    /// The session failed.
    Session(String),
}

impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused { eresult, message } => {
                let reason = describe_login_result(*eresult);
                match message {
                    Some(message) if !message.is_empty() => {
                        write!(f, "{reason}: {message} (EResult {eresult})")
                    }
                    _ => write!(f, "{reason} (EResult {eresult})"),
                }
            }
            Self::Password(message) => write!(f, "{message}"),
            Self::Incomplete(what) => write!(f, "Steam's response had no {what}"),
            Self::Session(message) => write!(f, "{message}"),
        }
    }
}

/// Says what a login result code means.
///
/// The number alone is useless to the person who has to act on it, and these
/// four are almost all of what a login actually returns. Anything else keeps
/// its number rather than being guessed at.
#[must_use]
pub fn describe_login_result(eresult: i32) -> &'static str {
    match eresult {
        5 => "wrong account name or password",
        15 => "Steam refused this account access",
        // 63 and 85 both mean "the account has Steam Guard on it".
        63 | 85 => "this account needs a Steam Guard code",
        65 => "that Steam Guard code was wrong",
        84 => {
            "too many attempts; Steam is rate limiting this account. Wait, \
               and prefer `--qr`, which is not throttled the same way"
        }
        88 => "that Steam Guard code has expired; ask for a new one",
        _ => "Steam refused the login",
    }
}

impl std::error::Error for LoginError {}

impl From<LoginError> for InstallError {
    fn from(error: LoginError) -> Self {
        Self::Io(error.to_string())
    }
}

/// Builds the device details Steam records against the login.
///
/// Named honestly, so someone auditing their account's authorised devices can
/// tell what added one.
#[must_use]
pub fn device_details()
-> tapline_proto::steammessages_auth_steamclient::CAuthentication_DeviceDetails {
    tapline_proto::steammessages_auth_steamclient::CAuthentication_DeviceDetails {
        device_friendly_name: Some(DEVICE_NAME.to_owned()),
        platform_type: Some(
            tapline_proto::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                PLATFORM_STEAM_CLIENT,
            ),
        ),
        // 1 = Linux, in EOSType terms Steam accepts here.
        os_type: Some(-203),
        ..Default::default()
    }
}

/// Turns Steam's allowed-confirmation list into ours.
#[must_use]
pub fn confirmations_from(
    allowed: &[tapline_proto::steammessages_auth_steamclient::CAuthentication_AllowedConfirmation],
) -> Vec<GuardType> {
    allowed
        .iter()
        .map(|entry| GuardType::from_i32(entry.confirmation_type.map_or(0, |kind| kind.value())))
        .collect()
}

/// Parses the RSA key from Steam's response.
pub fn key_from_response(
    response: &tapline_proto::steammessages_auth_steamclient::CAuthentication_GetPasswordRSAPublicKey_Response,
) -> Result<PublicKey, LoginError> {
    let modulus = response
        .publickey_mod
        .as_deref()
        .ok_or(LoginError::Incomplete("RSA modulus"))?;
    let exponent = response
        .publickey_exp
        .as_deref()
        .ok_or(LoginError::Incomplete("RSA exponent"))?;

    PublicKey::from_hex(modulus, exponent, response.timestamp.unwrap_or(0))
        .map_err(|error| LoginError::Password(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_proto::steammessages_auth_steamclient::{
        CAuthentication_AllowedConfirmation, CAuthentication_GetPasswordRSAPublicKey_Response,
    };

    #[test]
    fn a_refusal_says_what_to_do_about_it() {
        // "EResult 5" tells the person nothing; the number is for the log.
        let wrong = LoginError::Refused {
            eresult: 5,
            message: None,
        }
        .to_string();
        assert!(wrong.contains("password"), "{wrong}");
        assert!(
            wrong.contains("EResult 5"),
            "the code should still be there"
        );

        let throttled = LoginError::Refused {
            eresult: 84,
            message: None,
        }
        .to_string();
        assert!(throttled.contains("rate limiting"), "{throttled}");

        // An unknown code keeps its number rather than being invented.
        let odd = LoginError::Refused {
            eresult: 1234,
            message: None,
        }
        .to_string();
        assert!(odd.contains("1234"), "{odd}");
    }

    fn pending(url: Option<&str>, confirmations: Vec<GuardType>) -> PendingLogin {
        PendingLogin {
            steam_id: 0,
            client_id: 1,
            request_id: vec![1, 2, 3],
            interval: 5.0,
            confirmations,
            challenge_url: url.map(str::to_owned),
            account: None,
        }
    }

    #[test]
    fn a_qr_login_tells_the_user_to_scan() {
        let login = pending(
            Some("https://s.team/q/1/2"),
            vec![GuardType::DeviceConfirmation],
        );
        assert!(login.instruction().contains("https://s.team/q/1/2"));
        assert!(!login.requires_a_code(), "a QR login must not force a code");
    }

    #[test]
    fn a_real_qr_login_offers_a_code_as_an_alternative() {
        // Measured against live Steam: a QR session comes back offering
        // [DeviceConfirmation, DeviceCode]. The list is alternatives, not
        // requirements — reading it as requirements would prompt for a code
        // during a login that needs none.
        let login = pending(
            Some("https://s.team/q/1/2"),
            vec![GuardType::DeviceConfirmation, GuardType::DeviceCode],
        );

        assert!(login.accepts_a_code(), "the code alternative was lost");
        assert!(
            !login.requires_a_code(),
            "a QR login was treated as requiring a code"
        );
        // And the alternative is mentioned, for someone without their phone.
        assert!(login.instruction().contains("Steam Guard code"));
    }

    #[test]
    fn a_guard_code_login_says_where_the_code_comes_from() {
        // Email and authenticator are different places to look, and telling
        // someone the wrong one wastes their time.
        let email = pending(None, vec![GuardType::EmailCode]);
        assert!(email.requires_a_code());
        assert!(email.instruction().contains("email"));

        let device = pending(None, vec![GuardType::DeviceCode]);
        assert!(device.requires_a_code());
        assert!(device.instruction().contains("authenticator"));
    }

    #[test]
    fn an_approval_is_not_a_code_prompt() {
        let login = pending(None, vec![GuardType::DeviceConfirmation]);
        assert!(!login.accepts_a_code());
        assert!(!login.requires_a_code());
        assert!(login.instruction().contains("mobile app"));
    }

    #[test]
    fn confirmations_decode_in_the_order_steam_listed_them() {
        let allowed = vec![
            CAuthentication_AllowedConfirmation {
                confirmation_type: Some(
                    tapline_proto::steammessages_auth_steamclient::EAuthSessionGuardType::from(4),
                ),
                ..Default::default()
            },
            CAuthentication_AllowedConfirmation {
                confirmation_type: Some(
                    tapline_proto::steammessages_auth_steamclient::EAuthSessionGuardType::from(3),
                ),
                ..Default::default()
            },
        ];
        assert_eq!(
            confirmations_from(&allowed),
            vec![GuardType::DeviceConfirmation, GuardType::DeviceCode]
        );
    }

    #[test]
    fn a_missing_rsa_key_is_reported_as_missing() {
        // Rather than encrypting a password under a default-constructed key.
        let response = CAuthentication_GetPasswordRSAPublicKey_Response::default();
        assert_eq!(
            key_from_response(&response),
            Err(LoginError::Incomplete("RSA modulus"))
        );
    }

    #[test]
    fn a_refusal_keeps_steams_own_code_and_message() {
        // Wrong password, rate limited and needs-a-Guard-code lead to three
        // different next actions.
        let error = LoginError::Refused {
            eresult: 5,
            message: Some("Invalid password".to_owned()),
        };
        let rendered = error.to_string();
        assert!(rendered.contains('5'), "{rendered}");
        assert!(rendered.contains("Invalid password"), "{rendered}");
    }

    #[test]
    fn the_device_name_is_honest_about_what_it_is() {
        // Someone auditing their authorised devices should be able to tell what
        // added one.
        let details = device_details();
        assert_eq!(details.device_friendly_name.as_deref(), Some("tapline"));
    }
}
