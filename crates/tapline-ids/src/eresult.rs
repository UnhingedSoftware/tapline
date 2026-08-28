//! `EResult`: the status code on nearly every Steam response.

use std::fmt;

/// Generates the enum, wire conversions and descriptions from one table.
macro_rules! eresults {
    ($( $(#[$doc:meta])* $name:ident = $value:expr, $text:literal ;)*) => {
        /// A Steam result code.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum EResult {
            $( $(#[$doc])* $name, )*
            /// A code this build does not know about; the number is kept.
            Unknown(i32),
        }

        impl EResult {
            /// The wire encoding.
            #[must_use]
            pub const fn to_i32(self) -> i32 {
                match self {
                    $( Self::$name => $value, )*
                    Self::Unknown(v) => v,
                }
            }

            /// Decodes the wire encoding.
            #[must_use]
            pub const fn from_i32(v: i32) -> Self {
                match v {
                    $( $value => Self::$name, )*
                    other => Self::Unknown(other),
                }
            }

            /// A short description, for error messages.
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
    /// The request succeeded.
    Ok = 1, "success";
    /// Generic failure.
    Fail = 2, "generic failure";
    /// No connection to Steam.
    NoConnection = 3, "no connection to Steam";
    /// The password or ticket was wrong.
    InvalidPassword = 5, "invalid password or ticket";
    /// The account is logged in elsewhere.
    LoggedInElsewhere = 6, "logged in elsewhere";
    /// The protocol version is not supported.
    InvalidProtocolVer = 7, "unsupported protocol version";
    /// A parameter was invalid.
    InvalidParam = 8, "invalid parameter";
    /// The requested file was not found.
    FileNotFound = 9, "file not found";
    /// The account or resource is busy.
    Busy = 10, "busy";
    /// The state was invalid for this call.
    InvalidState = 11, "invalid state";
    /// The name was invalid.
    InvalidName = 12, "invalid name";
    /// The email was invalid.
    InvalidEmail = 13, "invalid email";
    /// The value duplicates an existing one.
    DuplicateName = 14, "duplicate name";
    /// Permission denied.
    AccessDenied = 15, "access denied";
    /// The operation timed out.
    Timeout = 16, "timed out";
    /// The account or IP is VAC banned.
    Banned = 17, "banned";
    /// No such account.
    AccountNotFound = 18, "account not found";
    /// The Steam ID was invalid.
    InvalidSteamID = 19, "invalid Steam ID";
    /// The service is unavailable.
    ServiceUnavailable = 20, "service unavailable";
    /// Not logged on.
    NotLoggedOn = 21, "not logged on";
    /// The request is pending.
    Pending = 22, "pending";
    /// Encryption or decryption failed.
    EncryptionFailure = 23, "encryption failure";
    /// Insufficient privilege.
    InsufficientPrivilege = 24, "insufficient privilege";
    /// A limit was exceeded.
    LimitExceeded = 25, "limit exceeded";
    /// The licence or guest pass has been revoked.
    Revoked = 26, "revoked";
    /// The licence or guest pass has expired.
    Expired = 27, "expired";
    /// Already redeemed.
    AlreadyRedeemed = 28, "already redeemed";
    /// A duplicate request.
    DuplicateRequest = 29, "duplicate request";
    /// The account already owns this.
    AlreadyOwned = 30, "already owned";
    /// The IP address was not found.
    IPNotFound = 31, "IP not found";
    /// Persisting the change failed.
    PersistFailed = 32, "failed to persist";
    /// A locking operation failed.
    LockingFailed = 33, "locking failed";
    /// The logon session was replaced.
    LogonSessionReplaced = 34, "logon session replaced";
    /// Connecting failed.
    ConnectFailed = 35, "connect failed";
    /// The handshake failed.
    HandshakeFailed = 36, "handshake failed";
    /// A generic IO failure.
    IOFailure = 37, "IO failure";
    /// The remote end disconnected.
    RemoteDisconnect = 38, "remote disconnect";
    /// The shopping cart was not found.
    ShoppingCartNotFound = 39, "shopping cart not found";
    /// The user blocked the action.
    Blocked = 40, "blocked";
    /// Steam Guard is required: a code was emailed.
    AccountLogonDenied = 63, "Steam Guard code required (emailed)";
    /// The password must be re-entered.
    RequirePasswordReEntry = 65, "password re-entry required";
    /// A value was out of range.
    ValueOutOfRange = 66, "value out of range";
    /// An unexpected error.
    UnexpectedError = 67, "unexpected error";
    /// The feature is disabled.
    Disabled = 68, "disabled";
    /// The region is locked.
    RegionLocked = 71, "region locked";
    /// Rate limited: back off.
    RateLimitExceeded = 84, "rate limit exceeded";
    /// A two-factor code is required.
    AccountLoginDeniedNeedTwoFactor = 85, "two-factor code required";
    /// The item or content was deleted.
    ItemDeleted = 86, "item deleted";
    /// Too many login attempts.
    AccountLoginDeniedThrottle = 87, "login throttled";
    /// The two-factor code did not match.
    TwoFactorCodeMismatch = 88, "two-factor code mismatch";
    /// Not modified since the given version.
    NotModified = 91, "not modified";
    /// The account is limited.
    AccountLimitExceeded = 95, "account limit exceeded";
    /// The account activity limit was exceeded.
    AccountActivityLimitExceeded = 96, "account activity limit exceeded";
    /// The IP is not allowed.
    IPBanned = 105, "IP banned";
    /// The token expired.
    AccessTokenExpired = 108, "access token expired";
    /// Try a different CM server.
    TryAnotherCM = 42, "try another CM";
    /// The disk is full.
    DiskFull = 50, "disk full";
    /// A remote call failed.
    RemoteCallFailed = 51, "remote call failed";
    /// The data was corrupt.
    DataCorruption = 49, "data corruption";
    /// The operation was cancelled.
    Cancelled = 48, "cancelled";
    /// The account is suspended.
    Suspended = 47, "suspended";
}

impl EResult {
    /// Whether the call actually worked; [`EResult::Pending`] is not ok.
    #[inline]
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Whether retrying the same request could plausibly succeed.
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
