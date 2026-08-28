use std::fmt;

macro_rules! eresults {
    ($( $(#[$doc:meta])* $name:ident = $value:expr, $text:literal ;)*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum EResult {
            $( $(#[$doc])* $name, )*
            Unknown(i32),
        }

        impl EResult {
            #[must_use]
            pub const fn to_i32(self) -> i32 {
                match self {
                    $( Self::$name => $value, )*
                    Self::Unknown(v) => v,
                }
            }

            #[must_use]
            pub const fn from_i32(v: i32) -> Self {
                match v {
                    $( $value => Self::$name, )*
                    other => Self::Unknown(other),
                }
            }

            #[must_use]
            pub const fn describe(self) -> &'static str {
                match self {
                    $( Self::$name => $text, )*
                    Self::Unknown(_) => "unknown result code",
                }
            }
        }
    };
}

eresults! {
    Ok = 1, "success";
    Fail = 2, "generic failure";
    NoConnection = 3, "no connection to Steam";
    InvalidPassword = 5, "invalid password or ticket";
    LoggedInElsewhere = 6, "logged in elsewhere";
    InvalidProtocolVer = 7, "unsupported protocol version";
    InvalidParam = 8, "invalid parameter";
    FileNotFound = 9, "file not found";
    Busy = 10, "busy";
    InvalidState = 11, "invalid state";
    InvalidName = 12, "invalid name";
    InvalidEmail = 13, "invalid email";
    DuplicateName = 14, "duplicate name";
    AccessDenied = 15, "access denied";
    Timeout = 16, "timed out";
    Banned = 17, "banned";
    AccountNotFound = 18, "account not found";
    InvalidSteamID = 19, "invalid Steam ID";
    ServiceUnavailable = 20, "service unavailable";
    NotLoggedOn = 21, "not logged on";
    Pending = 22, "pending";
    EncryptionFailure = 23, "encryption failure";
    InsufficientPrivilege = 24, "insufficient privilege";
    LimitExceeded = 25, "limit exceeded";
    Revoked = 26, "revoked";
    Expired = 27, "expired";
    AlreadyRedeemed = 28, "already redeemed";
    DuplicateRequest = 29, "duplicate request";
    AlreadyOwned = 30, "already owned";
    IPNotFound = 31, "IP not found";
    PersistFailed = 32, "failed to persist";
    LockingFailed = 33, "locking failed";
    LogonSessionReplaced = 34, "logon session replaced";
    ConnectFailed = 35, "connect failed";
    HandshakeFailed = 36, "handshake failed";
    IOFailure = 37, "IO failure";
    RemoteDisconnect = 38, "remote disconnect";
    ShoppingCartNotFound = 39, "shopping cart not found";
    Blocked = 40, "blocked";
    AccountLogonDenied = 63, "Steam Guard code required (emailed)";
    RequirePasswordReEntry = 65, "password re-entry required";
    ValueOutOfRange = 66, "value out of range";
    UnexpectedError = 67, "unexpected error";
    Disabled = 68, "disabled";
    RegionLocked = 71, "region locked";
    RateLimitExceeded = 84, "rate limit exceeded";
    AccountLoginDeniedNeedTwoFactor = 85, "two-factor code required";
    ItemDeleted = 86, "item deleted";
    AccountLoginDeniedThrottle = 87, "login throttled";
    TwoFactorCodeMismatch = 88, "two-factor code mismatch";
    NotModified = 91, "not modified";
    AccountLimitExceeded = 95, "account limit exceeded";
    AccountActivityLimitExceeded = 96, "account activity limit exceeded";
    IPBanned = 105, "IP banned";
    AccessTokenExpired = 108, "access token expired";
    TryAnotherCM = 42, "try another CM";
    DiskFull = 50, "disk full";
    RemoteCallFailed = 51, "remote call failed";
    DataCorruption = 49, "data corruption";
    Cancelled = 48, "cancelled";
    Suspended = 47, "suspended";
}

impl EResult {
    #[inline]
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    #[must_use]
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::NoConnection
                | Self::Busy
                | Self::Timeout
                | Self::ServiceUnavailable
                | Self::TryAnotherCM
                | Self::RateLimitExceeded
                | Self::IOFailure
                | Self::RemoteDisconnect
                | Self::ConnectFailed
                | Self::Pending
        )
    }
}

impl fmt::Display for EResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.describe(), self.to_i32())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codes_round_trip() {
        for code in [
            EResult::Ok,
            EResult::AccessDenied,
            EResult::TwoFactorCodeMismatch,
        ] {
            assert_eq!(EResult::from_i32(code.to_i32()), code);
        }
    }

    #[test]
    fn unknown_codes_keep_their_number() {
        let r = EResult::from_i32(9_999);
        assert_eq!(r, EResult::Unknown(9_999));
        assert_eq!(r.to_i32(), 9_999);
        assert!(format!("{r}").contains("9999"));
    }

    #[test]
    fn pending_is_not_success() {
        assert!(!EResult::Pending.is_ok());
        assert!(EResult::Pending.is_retryable());
    }

    #[test]
    fn permanent_failures_are_not_retried() {
        assert!(!EResult::InvalidPassword.is_retryable());
        assert!(!EResult::AccessDenied.is_retryable());
        assert!(!EResult::Banned.is_retryable());
    }
}
