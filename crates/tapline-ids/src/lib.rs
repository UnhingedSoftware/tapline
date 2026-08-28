mod eresult;
mod steamid;

pub use eresult::EResult;
pub use steamid::{AccountType, SteamId, Universe};

use std::fmt;

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident($inner:ty)) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        #[repr(transparent)]
        pub struct $name(pub $inner);

        impl $name {
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
    AppId(u32)
}

id_newtype! {
    DepotId(u32)
}

id_newtype! {
    ManifestId(u64)
}

id_newtype! {
    PublishedFileId(u64)
}

id_newtype! {
    PackageId(u32)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DepotKey([u8; 32]);

impl DepotKey {
    #[inline]
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

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
