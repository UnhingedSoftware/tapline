//! Where a refresh token lives between runs.

use std::fmt;
use std::path::{Path, PathBuf};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A stored refresh token; zeroed on drop, never printed.
#[derive(Clone, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
pub struct StoredToken {
    /// The account the token belongs to.
    pub account: String,
    /// The refresh token itself.
    pub refresh_token: String,
}

impl StoredToken {
    /// The SteamID from the token's `sub` claim; `None` means "log in again".
    #[must_use]
    pub fn steam_id(&self) -> Option<u64> {
        steam_id_from_jwt(&self.refresh_token)
    }
}

/// Reads the JWT `sub` claim without verifying; Steam validates the token itself.
fn steam_id_from_jwt(token: &str) -> Option<u64> {
    // header.payload.signature — the middle part is the claims.
    let payload = token.split('.').nth(1)?;
    let decoded = base64url_decode(payload)?;
    let text = std::str::from_utf8(&decoded).ok()?;

    // Targeted scan; one field does not justify a JSON parser.
    let start = text.find("\"sub\"")?;
    let rest = text.get(start + 5..)?;
    let quoted = rest.find('"')?;
    let digits: String = rest
        .get(quoted + 1..)?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Decodes unpadded base64url, which is what a JWT uses.
fn base64url_decode(input: &str) -> Option<Vec<u8>> {
    fn value(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'-' => Some(62),
            b'_' => Some(63),
            _ => None,
        }
    }

    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0_u32;
    for byte in input.bytes() {
        // JWTs are unpadded, but tolerate padding rather than failing on it.
        if byte == b'=' {
            break;
        }
        accumulator = (accumulator << 6) | u32::from(value(byte)?);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((accumulator >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

impl fmt::Debug for StoredToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredToken")
            .field("account", &self.account)
            .field("refresh_token", &"<redacted>")
            .finish()
    }
}

/// What went wrong reading or writing a token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenStoreError {
    /// The filesystem or keyring refused.
    Backend(String),
    /// The stored data was not in the expected shape.
    Malformed,
    /// The token file is readable by other users; refused rather than used.
    Insecure {
        /// The file's mode.
        mode: u32,
    },
}

impl fmt::Display for TokenStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Backend(message) => write!(f, "{message}"),
            Self::Malformed => f.write_str("the stored token is not in the expected format"),
            Self::Insecure { mode } => write!(
                f,
                "the token file is mode {mode:o}; it must not be readable by other users"
            ),
        }
    }
}

impl std::error::Error for TokenStoreError {}

/// Where tokens are kept.
#[derive(Debug, Clone, Default)]
pub enum TokenStore {
    /// A `0600` file.
    File {
        /// Its path.
        path: PathBuf,
    },
    /// The OS keyring.
    #[cfg(feature = "keyring")]
    Keyring,
    /// Nowhere: the token is discarded when the process ends.
    #[default]
    None,
}

#[cfg(feature = "keyring")]
const KEYRING_SERVICE: &str = "tapline";

impl TokenStore {
    /// A file store under the user's config directory.
    #[must_use]
    pub fn default_file() -> Self {
        let base = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config")
            });
        Self::File {
            path: base.join("tapline").join("tokens"),
        }
    }

    /// Saves a token.
    pub fn save(&self, token: &StoredToken) -> Result<(), TokenStoreError> {
        match self {
            Self::None => Ok(()),
            Self::File { path } => write_file(path, token),
            #[cfg(feature = "keyring")]
            Self::Keyring => {
                let entry = keyring::Entry::new(KEYRING_SERVICE, &token.account)
                    .map_err(|e| TokenStoreError::Backend(e.to_string()))?;
                entry
                    .set_password(&token.refresh_token)
                    .map_err(|e| TokenStoreError::Backend(e.to_string()))
            }
        }
    }

    /// Loads an account's token, if one is stored.
    pub fn load(&self, account: &str) -> Result<Option<StoredToken>, TokenStoreError> {
        match self {
            Self::None => Ok(None),
            Self::File { path } => read_file(path, account),
            #[cfg(feature = "keyring")]
            Self::Keyring => {
                let entry = keyring::Entry::new(KEYRING_SERVICE, account)
                    .map_err(|e| TokenStoreError::Backend(e.to_string()))?;
                match entry.get_password() {
                    Ok(refresh_token) => Ok(Some(StoredToken {
                        account: account.to_owned(),
                        refresh_token,
                    })),
                    Err(keyring::Error::NoEntry) => Ok(None),
                    Err(error) => Err(TokenStoreError::Backend(error.to_string())),
                }
            }
        }
    }

    /// The accounts with a saved token; the keyring backend returns empty.
    pub fn accounts(&self) -> Result<Vec<String>, TokenStoreError> {
        match self {
            Self::None => Ok(Vec::new()),
            Self::File { path } => Ok(read_all(path)?.into_iter().map(|(name, _)| name).collect()),
            #[cfg(feature = "keyring")]
            Self::Keyring => Ok(Vec::new()),
        }
    }

    /// Forgets every saved token; file backend only.
    pub fn forget_all(&self) -> Result<(), TokenStoreError> {
        match self {
            Self::None => Ok(()),
            Self::File { path } => write_all(path, &[]),
            #[cfg(feature = "keyring")]
            Self::Keyring => Ok(()),
        }
    }

    /// Forgets one account's token locally; it does not revoke with Steam.
    pub fn forget(&self, account: &str) -> Result<(), TokenStoreError> {
        match self {
            Self::None => Ok(()),
            Self::File { path } => {
                let mut entries = read_all(path)?;
                entries.retain(|(name, _)| name != account);
                write_all(path, &entries)
            }
            #[cfg(feature = "keyring")]
            Self::Keyring => {
                let entry = keyring::Entry::new(KEYRING_SERVICE, account)
                    .map_err(|e| TokenStoreError::Backend(e.to_string()))?;
                match entry.delete_credential() {
                    Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
                    Err(error) => Err(TokenStoreError::Backend(error.to_string())),
                }
            }
        }
    }
}

fn read_all(path: &Path) -> Result<Vec<(String, String)>, TokenStoreError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(TokenStoreError::Backend(error.to_string())),
    };

    check_permissions(path)?;

    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // `account\ttoken`; a tab needs no escaping in either field.
        let (account, token) = line.split_once('\t').ok_or(TokenStoreError::Malformed)?;
        out.push((account.to_owned(), token.to_owned()));
    }
    Ok(out)
}

fn write_all(path: &Path, entries: &[(String, String)]) -> Result<(), TokenStoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| TokenStoreError::Backend(e.to_string()))?;
    }

    let mut body = String::new();
    for (account, token) in entries {
        body.push_str(account);
        body.push('\t');
        body.push_str(token);
        body.push('\n');
    }

    create_private(path, &body)?;
    body.zeroize();
    Ok(())
}

/// Creates the file with `0600` before writing, so no world-readable window exists.
fn create_private(path: &Path, contents: &str) -> Result<(), TokenStoreError> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;

    // Temp file + rename, so a crash cannot truncate the existing file.
    let temporary = path.with_extension("tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|e| TokenStoreError::Backend(e.to_string()))?;

    file.write_all(contents.as_bytes())
        .map_err(|e| TokenStoreError::Backend(e.to_string()))?;
    file.sync_all()
        .map_err(|e| TokenStoreError::Backend(e.to_string()))?;
    drop(file);

    std::fs::rename(&temporary, path).map_err(|e| TokenStoreError::Backend(e.to_string()))
}

fn check_permissions(path: &Path) -> Result<(), TokenStoreError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|e| TokenStoreError::Backend(e.to_string()))?;
    let mode = metadata.permissions().mode() & 0o777;

    // Group or other having any access at all.
    if mode & 0o077 != 0 {
        return Err(TokenStoreError::Insecure { mode });
    }
    Ok(())
}

fn write_file(path: &Path, token: &StoredToken) -> Result<(), TokenStoreError> {
    let mut entries = read_all(path)?;
    entries.retain(|(account, _)| account != &token.account);
    entries.push((token.account.clone(), token.refresh_token.clone()));
    write_all(path, &entries)
}

fn read_file(path: &Path, account: &str) -> Result<Option<StoredToken>, TokenStoreError> {
    Ok(read_all(path)?
        .into_iter()
        .find(|(name, _)| name == account)
        .map(|(account, refresh_token)| StoredToken {
            account,
            refresh_token,
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// header.payload.signature, unpadded base64url, with the SteamID in `sub`.
    const REAL_SHAPE: &str = "eyJhbGciOiJFZERTQSJ9.eyJpc3MiOiAic3RlYW0iLCAic3ViIjogIjc2NTYxMTk4MTYwNTcwMjA4IiwgImF1ZCI6IFsiY2xpZW50IiwgIndlYiJdLCAiZXhwIjogMTkwMDAwMDAwMH0.c2lnbmF0dXJl";

    #[test]
    fn a_token_knows_which_account_it_is_for() {
        let token = StoredToken {
            account: "someone".to_owned(),
            refresh_token: REAL_SHAPE.to_owned(),
        };
        assert_eq!(token.steam_id(), Some(76_561_198_160_570_208));
    }

    #[test]
    fn rubbish_is_no_id_rather_than_a_panic() {
        for bad in [
            "",
            "not-a-jwt",
            "only.two",
            "a.!!!!.c",
            "a.eyJzdWIiOiJub3QtYS1udW1iZXIifQ.c",
        ] {
            let token = StoredToken {
                account: "someone".to_owned(),
                refresh_token: bad.to_owned(),
            };
            assert_eq!(token.steam_id(), None, "{bad:?} should not yield an id");
        }
    }

    #[test]
    fn base64url_reads_the_alphabet_a_jwt_uses() {
        // `-`/`_` alphabet, no padding; a standard decoder rejects these.
        assert_eq!(base64url_decode("Zm9vYmFy"), Some(b"foobar".to_vec()));
        assert_eq!(base64url_decode("Zg"), Some(b"f".to_vec()));
        assert_eq!(base64url_decode("Zg=="), Some(b"f".to_vec()));
        assert_eq!(
            base64url_decode("++"),
            None,
            "the standard alphabet is not this one"
        );
    }
    use std::os::unix::fs::PermissionsExt;

    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let base = std::env::var("TAPLINE_TEST_DIR").map_or_else(
                |_| {
                    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                        .join(".cache/tapline-test")
                },
                PathBuf::from,
            );
            let path = base.join(name);
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("scratch");
            Self(path)
        }

        fn store(&self) -> TokenStore {
            TokenStore::File {
                path: self.0.join("tokens"),
            }
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn token(account: &str) -> StoredToken {
        StoredToken {
            account: account.to_owned(),
            refresh_token: "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.fake".to_owned(),
        }
    }

    #[test]
    fn a_token_round_trips() {
        let scratch = Scratch::new("tokens-roundtrip");
        let store = scratch.store();

        assert_eq!(store.load("someone").expect("load"), None);
        store.save(&token("someone")).expect("save");
        assert_eq!(store.load("someone").expect("load"), Some(token("someone")));
    }

    #[test]
    fn accounts_lists_every_saved_login() {
        let scratch = Scratch::new("tokens-accounts");
        let store = scratch.store();
        store.save(&token("one")).expect("save one");
        store.save(&token("two")).expect("save two");
        let mut names = store.accounts().expect("accounts");
        names.sort();
        assert_eq!(names, vec!["one".to_owned(), "two".to_owned()]);
    }

    #[test]
    fn forget_all_clears_the_store() {
        let scratch = Scratch::new("tokens-forget-all");
        let store = scratch.store();
        store.save(&token("gone")).expect("save");
        store.forget_all().expect("forget all");
        assert!(store.accounts().expect("accounts").is_empty());
    }

    #[test]
    fn the_file_is_created_private_and_never_widens() {
        // The ordering is the point: 0600 at creation, not chmod afterwards.
        let scratch = Scratch::new("tokens-perms");
        let store = scratch.store();
        store.save(&token("someone")).expect("save");

        let path = scratch.0.join("tokens");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the token file is mode {mode:o}");

        // A second save must not widen it.
        store.save(&token("another")).expect("save");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn a_world_readable_token_file_is_refused_rather_than_used() {
        let scratch = Scratch::new("tokens-insecure");
        let store = scratch.store();
        store.save(&token("someone")).expect("save");

        let path = scratch.0.join("tokens");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");

        assert!(matches!(
            store.load("someone"),
            Err(TokenStoreError::Insecure { mode: 0o644 })
        ));
    }

    #[test]
    fn several_accounts_coexist() {
        let scratch = Scratch::new("tokens-multi");
        let store = scratch.store();

        store.save(&token("first")).expect("save");
        store.save(&token("second")).expect("save");

        assert!(store.load("first").expect("load").is_some());
        assert!(store.load("second").expect("load").is_some());
    }

    #[test]
    fn saving_the_same_account_twice_replaces_rather_than_appends() {
        let scratch = Scratch::new("tokens-replace");
        let store = scratch.store();

        store.save(&token("someone")).expect("save");
        store
            .save(&StoredToken {
                account: "someone".to_owned(),
                refresh_token: "a-newer-token".to_owned(),
            })
            .expect("save");

        let loaded = store.load("someone").expect("load").expect("present");
        assert_eq!(loaded.refresh_token, "a-newer-token");

        let text = std::fs::read_to_string(scratch.0.join("tokens")).expect("read");
        assert_eq!(text.lines().count(), 1, "the old entry was left behind");
    }

    #[test]
    fn forgetting_removes_only_that_account() {
        let scratch = Scratch::new("tokens-forget");
        let store = scratch.store();

        store.save(&token("first")).expect("save");
        store.save(&token("second")).expect("save");
        store.forget("first").expect("forget");

        assert_eq!(store.load("first").expect("load"), None);
        assert!(store.load("second").expect("load").is_some());
    }

    #[test]
    fn the_default_store_keeps_nothing() {
        let store = TokenStore::default();
        store.save(&token("someone")).expect("save");
        assert_eq!(store.load("someone").expect("load"), None);
    }

    #[test]
    fn a_token_does_not_print_itself() {
        let rendered = format!("{:?}", token("someone"));
        assert!(rendered.contains("someone"));
        assert!(rendered.contains("<redacted>"));
        assert!(
            !rendered.contains("eyJhbGciOiJFZERTQSIs"),
            "the token leaked into Debug output"
        );
    }

    #[test]
    fn a_malformed_file_is_reported_rather_than_half_read() {
        let scratch = Scratch::new("tokens-malformed");
        let path = scratch.0.join("tokens");
        create_private(&path, "this line has no tab\n").expect("write");

        assert_eq!(
            scratch.store().load("someone"),
            Err(TokenStoreError::Malformed)
        );
    }
}
