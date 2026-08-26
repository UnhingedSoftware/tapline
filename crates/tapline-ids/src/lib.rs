//! Steam's identifier types.
//!
//! These are newtypes rather than bare `u32`/`u64` because the protocol hands
//! you half a dozen different integers that all look alike on the wire, and
//! passing a depot id where a manifest id belongs is otherwise a silent bug that
//! surfaces as an HTTP 401 from the CDN three layers away.
//!
//! The crate has no dependencies, internal or external. It is not `no_std`:
//! browsers are out of scope, so that would buy portability nobody asked for at
//! the cost of friction in every test.

mod eresult;
mod steamid;

pub use eresult::EResult;
pub use steamid::{AccountType, SteamId, Universe};

use std::fmt;

/// Declares a transparent integer newtype with the conversions and formatting
/// every id in this crate wants, and nothing else.
macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident($inner:ty)) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[repr(transparent)]
        pub struct $name(pub $inner);

        impl $name {
            /// The underlying integer, as it appears on the wire.
            #[inline]
            #[must_use]
            pub const fn get(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            #[inline]
            fn from(v: $inner) -> Self {
                Self(v)
            }
        }

        impl From<$name> for $inner {
            #[inline]
            fn from(v: $name) -> Self {
                v.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

id_newtype! {
    /// An application: a game, a tool, or a dedicated server.
    ///
    /// Dedicated servers have their own appid distinct from the game's — Team
    /// Fortress 2 is 440 but its server is 232250 — which is why installing a
    /// server never needs the game to be owned.
    AppId(u32)
}

id_newtype! {
    /// A depot: one bucket of content belonging to an app, usually split by
    /// platform, architecture or language.
    DepotId(u32)
}

id_newtype! {
    /// A specific build of a depot's contents.
    ///
    /// This is the thing that pins an install to an exact set of bytes, and the
    /// only way to reproduce an old build.
    ManifestId(u64)
}

id_newtype! {
    /// A Workshop item.
    PublishedFileId(u64)
}

id_newtype! {
    /// A licence-granting package. Accounts own packages; packages contain apps.
    PackageId(u32)
}

/// A depot's AES-256 decryption key, as handed out by Steam.
///
/// Wrapped rather than passed as a bare `[u8; 32]` so it cannot be logged by
/// accident: the [`fmt::Debug`] impl deliberately prints nothing useful. Steam
/// grants these only for depots the signed-in account is entitled to, and that
/// entitlement check is the whole reason this crate never learned to cache them
/// to disk.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DepotKey([u8; 32]);

impl DepotKey {
    /// Wraps 32 key bytes.
    #[inline]
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw key, for handing to a cipher.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for DepotKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DepotKey(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtypes_round_trip_through_their_integer() {
        assert_eq!(u32::from(AppId::from(232_250_u32)), 232_250);
        assert_eq!(ManifestId::from(7_u64).get(), 7);
    }

    #[test]
    fn depot_key_does_not_leak_through_debug() {
        // The point of the wrapper: a key that reaches a tracing span must not
        // print itself.
        let key = DepotKey::new([0xAB; 32]);
        let rendered = format!("{key:?}");
        assert_eq!(rendered, "DepotKey(<redacted>)");
        assert!(
            !rendered.contains("ab"),
            "key bytes leaked into Debug output"
        );
    }
}
