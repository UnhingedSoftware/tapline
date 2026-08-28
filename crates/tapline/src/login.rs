use crate::InstallError;
use std::fmt;
use tapline_auth::{DEVICE_NAME, GuardType, PLATFORM_STEAM_CLIENT, PublicKey};

#[derive(Debug, Clone)]
pub struct PendingLogin {
    pub client_id: u64,
    pub request_id: Vec<u8>,
    pub interval: f32,
    pub confirmations: Vec<GuardType>,
    pub challenge_url: Option<String>,
    pub account: Option<String>,
    pub steam_id: u64,
}

impl PendingLogin {
    #[must_use]
    pub fn accepts_a_code(&self) -> bool {
        self.confirmations.iter().any(|guard| guard.needs_a_code())
    }

    #[must_use]
    pub fn requires_a_code(&self) -> bool {
        self.challenge_url.is_none()
            && !self.confirmations.is_empty()
            && self.confirmations.iter().all(|guard| guard.needs_a_code())
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollOutcome {
    Pending {
        had_interaction: bool,
    },
    Moved {
        client_id: u64,
        challenge_url: Option<String>,
    },
    Complete {
        account: String,
        refresh_token: String,
        access_token: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginError {
    Refused {
        eresult: i32,
        message: Option<String>,
    },
    Password(String),
    Incomplete(&'static str),
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

#[must_use]
pub fn describe_login_result(eresult: i32) -> &'static str {
    match eresult {
        5 => "wrong account name or password",
        15 => "Steam refused this account access",
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
        os_type: Some(-203),
        ..Default::default()
    }
}

#[must_use]
pub fn confirmations_from(
    allowed: &[tapline_proto::steammessages_auth_steamclient::CAuthentication_AllowedConfirmation],
) -> Vec<GuardType> {
    allowed
        .iter()
        .map(|entry| GuardType::from_i32(entry.confirmation_type.map_or(0, |kind| kind.value())))
        .collect()
}

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
        let login = pending(
            Some("https://s.team/q/1/2"),
            vec![GuardType::DeviceConfirmation, GuardType::DeviceCode],
        );

        assert!(login.accepts_a_code(), "the code alternative was lost");
        assert!(
            !login.requires_a_code(),
            "a QR login was treated as requiring a code"
        );
        assert!(login.instruction().contains("Steam Guard code"));
    }

    #[test]
    fn a_guard_code_login_says_where_the_code_comes_from() {
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
        let response = CAuthentication_GetPasswordRSAPublicKey_Response::default();
        assert_eq!(
            key_from_response(&response),
            Err(LoginError::Incomplete("RSA modulus"))
        );
    }

    #[test]
    fn a_refusal_keeps_steams_own_code_and_message() {
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
        let details = device_details();
        assert_eq!(details.device_friendly_name.as_deref(), Some("tapline"));
    }
}
