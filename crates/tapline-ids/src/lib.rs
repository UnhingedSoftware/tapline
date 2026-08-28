//! Steam's identifier types, as newtypes so ids cannot be swapped silently.

mod eresult;
mod steamid;

pub use eresult::EResult;
pub use steamid::{AccountType, SteamId, Universe};

use std::fmt;

/// Declares a transparent integer id newtype with conversions and formatting.
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
    AppId(u32)
}

id_newtype! {
    /// One bucket of an app's content, split by platform, architecture or language.
    DepotId(u32)
}

id_newtype! {
    /// A specific build of a depot's contents; pins an install to exact bytes.
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

/// A depot's AES-256 key; `Debug` prints nothing so it cannot leak into logs.
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
        let key = DepotKey::new([0xAB; 32]);
        let rendered = format!("{key:?}");
        assert_eq!(rendered, "DepotKey(<redacted>)");
        assert!(
            !rendered.contains("ab"),
            "key bytes leaked into Debug output"
        );
    }
}
