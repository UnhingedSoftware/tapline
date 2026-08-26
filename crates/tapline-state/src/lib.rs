//! The install record: `steamapps/appmanifest_<appid>.acf`.
//!
//! This file is why tapline can take over an install steamcmd made, and why an
//! install tapline makes is one the Steam client, steamcmd, LinuxGSM and every
//! host panel already understand. It records which depots are installed at which
//! manifest ids — which is exactly what a delta update diffs against.
//!
//! # Round-tripping is the point
//!
//! Reading and rewriting one of these must change only the fields that changed.
//! Fields tapline does not model are preserved verbatim rather than dropped,
//! because Valve adds them and an install that lost `ScheduledAutoUpdate` on
//! every update would be quietly degrading a file other tools read.
//!
//! Verified against `appmanifest_896660.acf` exactly as steamcmd wrote it while
//! installing Valheim Dedicated Server on 2026-08-26: parse, rewrite, and the
//! bytes are identical.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use tapline_ids::{AppId, DepotId, ManifestId};
use tapline_vdf::{Object, Value, VdfError};

/// `StateFlags` values, as Valve sets them.
pub mod state_flags {
    /// The install is complete and usable.
    pub const FULLY_INSTALLED: u32 = 4;
    /// An update is needed.
    pub const UPDATE_REQUIRED: u32 = 2;
    /// Files are being downloaded.
    pub const UPDATE_RUNNING: u32 = 1024;
}

/// One installed depot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstalledDepot {
    /// The build that is on disk.
    pub manifest: ManifestId,
    /// Its installed size in bytes.
    pub size: u64,
}

/// What went wrong reading or writing an install record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// The file did not parse.
    Malformed(VdfError),
    /// The document had no `AppState` block.
    NotAnAppManifest,
    /// The file names a different app than the one asked for.
    ///
    /// A mismatch here means the install directory holds someone else's app, and
    /// writing over it would be worse than failing.
    WrongApp {
        /// What was expected.
        expected: AppId,
        /// What the file says.
        found: AppId,
    },
    /// The filesystem refused.
    Io(String),
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(e) => write!(f, "the appmanifest did not parse: {e}"),
            Self::NotAnAppManifest => f.write_str("the file has no AppState block"),
            Self::WrongApp { expected, found } => {
                write!(
                    f,
                    "expected an appmanifest for app {expected}, found {found}"
                )
            }
            Self::Io(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for StateError {}

impl From<VdfError> for StateError {
    fn from(error: VdfError) -> Self {
        Self::Malformed(error)
    }
}

impl From<std::io::Error> for StateError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

/// An app's install record.
///
/// Holds the whole document, so fields tapline does not model survive a
/// rewrite. The typed accessors read and write through to it rather than
/// shadowing it, which is what keeps the two from drifting apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    document: Object,
}

impl AppState {
    /// The path of an app's record within an install root.
    #[must_use]
    pub fn path_for(root: &Path, app: AppId) -> PathBuf {
        root.join("steamapps")
            .join(format!("appmanifest_{app}.acf"))
    }

    /// Parses an install record.
    pub fn parse(text: &str, expected: AppId) -> Result<Self, StateError> {
        let document = tapline_vdf::parse(text)?;
        let state = Self { document };

        let found = state.app_id().ok_or(StateError::NotAnAppManifest)?;
        if found != expected {
            return Err(StateError::WrongApp { expected, found });
        }
        Ok(state)
    }

    /// Reads an app's record from an install root, if there is one.
    ///
    /// A missing file is `Ok(None)` rather than an error: a fresh install has no
    /// record yet, and that is not a failure.
    pub fn read(root: &Path, app: AppId) -> Result<Option<Self>, StateError> {
        let path = Self::path_for(root, app);
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(Some(Self::parse(&text, app)?)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Builds a fresh record for an app.
    pub fn new(app: AppId, name: &str, install_dir: &str) -> Self {
        let mut state = Object::new();
        // Written in the order Valve writes them, so a diff against a
        // steamcmd-made file shows only values.
        state.set_str("appid", app.to_string());
        state.set_str("Universe", "1");
        state.set_str("name", name);
        state.set_str("StateFlags", state_flags::FULLY_INSTALLED.to_string());
        state.set_str("installdir", install_dir);
        state.set_str("LastUpdated", "0");
        state.set_str("LastPlayed", "0");
        state.set_str("SizeOnDisk", "0");
        state.set_str("StagingSize", "0");
        state.set_str("buildid", "0");
        state.set_str("LastOwner", "0");
        state.set("InstalledDepots", Value::Object(Object::new()));
        state.set("UserConfig", Value::Object(Object::new()));
        state.set("MountedConfig", Value::Object(Object::new()));

        let mut document = Object::new();
        document.push("AppState", Value::Object(state));
        Self { document }
    }

    /// Writes the record, creating `steamapps/` if needed.
    ///
    /// Written to a temporary file and renamed, so a crash midway leaves the
    /// previous record intact rather than a half-written one. A truncated
    /// appmanifest is worse than a stale one: the Steam client treats it as a
    /// broken install.
    pub fn write(&self, root: &Path, app: AppId) -> Result<(), StateError> {
        let path = Self::path_for(root, app);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temporary = path.with_extension("acf.tmp");
        std::fs::write(&temporary, self.to_string())?;
        std::fs::rename(&temporary, &path)?;
        Ok(())
    }

    /// The `AppState` block.
    fn state(&self) -> Option<&Object> {
        self.document.get_object("AppState")
    }

    /// The `AppState` block, for modification.
    fn state_mut(&mut self) -> Object {
        self.document
            .get_object("AppState")
            .cloned()
            .unwrap_or_default()
    }

    /// Replaces the `AppState` block.
    fn put_state(&mut self, state: Object) {
        self.document.set("AppState", Value::Object(state));
    }

    /// The app this describes.
    #[must_use]
    pub fn app_id(&self) -> Option<AppId> {
        let value = self.state()?.get_u64("appid")?;
        u32::try_from(value).ok().map(AppId)
    }

    /// The app's name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.state()?.get_str("name")
    }

    /// The build on disk.
    #[must_use]
    pub fn build_id(&self) -> Option<u64> {
        self.state()?.get_u64("buildid")
    }

    /// The `StateFlags` word.
    #[must_use]
    pub fn state_flags(&self) -> u32 {
        self.state()
            .and_then(|state| state.get_u64("StateFlags"))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(0)
    }

    /// Whether the install is complete.
    #[must_use]
    pub fn is_fully_installed(&self) -> bool {
        self.state_flags() & state_flags::FULLY_INSTALLED != 0
    }

    /// The depots on disk and the builds they are at.
    #[must_use]
    pub fn installed_depots(&self) -> BTreeMap<DepotId, InstalledDepot> {
        let mut out = BTreeMap::new();
        let Some(depots) = self.state().and_then(|s| s.get_object("InstalledDepots")) else {
            return out;
        };

        for (key, value) in depots.iter() {
            let (Ok(id), Some(entry)) = (key.parse::<u32>(), value.as_object()) else {
                continue;
            };
            let Some(manifest) = entry.get_u64("manifest") else {
                continue;
            };
            out.insert(
                DepotId(id),
                InstalledDepot {
                    manifest: ManifestId(manifest),
                    size: entry.get_u64("size").unwrap_or(0),
                },
            );
        }
        out
    }

    /// The manifest a depot is installed at, if it is.
    ///
    /// This is the question a delta update asks: same id means nothing to do,
    /// a different id means diff the two manifests, absent means fetch it all.
    #[must_use]
    pub fn installed_manifest(&self, depot: DepotId) -> Option<ManifestId> {
        self.installed_depots().get(&depot).map(|d| d.manifest)
    }

    /// Records a depot as installed at a manifest.
    pub fn set_depot(&mut self, depot: DepotId, manifest: ManifestId, size: u64) {
        let mut state = self.state_mut();
        let mut depots = state
            .get_object("InstalledDepots")
            .cloned()
            .unwrap_or_default();

        let mut entry = depots
            .get_object(&depot.to_string())
            .cloned()
            .unwrap_or_default();
        entry.set_str("manifest", manifest.to_string());
        entry.set_str("size", size.to_string());

        depots.set(&depot.to_string(), Value::Object(entry));
        state.set("InstalledDepots", Value::Object(depots));
        self.put_state(state);
    }

    /// Removes a depot from the record.
    ///
    /// Used when an app stops shipping one: leaving it listed would make the
    /// next update think content is present that is not.
    pub fn remove_depot(&mut self, depot: DepotId) {
        let mut state = self.state_mut();
        let Some(existing) = state.get_object("InstalledDepots") else {
            return;
        };

        let mut depots = Object::new();
        let target = depot.to_string();
        for (key, value) in existing.iter() {
            if key != target {
                depots.push(key, value.clone());
            }
        }
        state.set("InstalledDepots", Value::Object(depots));
        self.put_state(state);
    }

    /// Sets a scalar field, adding it if absent.
    pub fn set_field(&mut self, key: &str, value: &str) {
        let mut state = self.state_mut();
        state.set_str(key, value);
        self.put_state(state);
    }

    /// Records the sizes and build an install finished at.
    pub fn mark_installed(&mut self, build_id: u64, size_on_disk: u64, updated_at: u64) {
        self.set_field("StateFlags", &state_flags::FULLY_INSTALLED.to_string());
        self.set_field("buildid", &build_id.to_string());
        self.set_field("TargetBuildID", &build_id.to_string());
        self.set_field("SizeOnDisk", &size_on_disk.to_string());
        self.set_field("LastUpdated", &updated_at.to_string());
        self.set_field("StagingSize", "0");
        self.set_field("UpdateResult", "0");
    }
}

impl fmt::Display for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// steamcmd's own output for Valheim Dedicated Server, 2026-08-26.
    const REAL: &str = include_str!("../tests/fixtures/appmanifest_896660.acf");

    fn valheim() -> AppState {
        AppState::parse(REAL, AppId(896_660)).expect("steamcmd's own file must parse")
    }

    #[test]
    fn steamcmds_own_record_round_trips_byte_for_byte() {
        // The gate. tapline shares this file with the Steam client and every
        // panel that greps it; a reformatted rewrite is a spurious diff for all
        // of them.
        assert_eq!(valheim().to_string(), REAL);
    }

    #[test]
    fn the_fields_an_update_turns_on_are_readable() {
        let state = valheim();
        assert_eq!(state.app_id(), Some(AppId(896_660)));
        assert_eq!(state.name(), Some("Valheim Dedicated Server"));
        assert_eq!(state.build_id(), Some(21_981_590));
        assert!(state.is_fully_installed());

        let depots = state.installed_depots();
        assert_eq!(depots.len(), 2);
        assert_eq!(
            state.installed_manifest(DepotId(896_661)),
            Some(ManifestId(962_159_520_942_340_660))
        );
        assert_eq!(
            state.installed_manifest(DepotId(1006)).map(|m| m.get()),
            Some(6_403_079_453_713_498_174)
        );
        // A depot that is not installed must be absent, not zero.
        assert_eq!(state.installed_manifest(DepotId(999_999)), None);
    }

    #[test]
    fn a_record_for_a_different_app_is_refused() {
        // The install directory holds someone else's app; writing over it would
        // be worse than failing.
        assert_eq!(
            AppState::parse(REAL, AppId(232_250)),
            Err(StateError::WrongApp {
                expected: AppId(232_250),
                found: AppId(896_660),
            })
        );
    }

    #[test]
    fn updating_a_depot_changes_only_that_depots_lines() {
        let mut state = valheim();
        state.set_depot(
            DepotId(896_661),
            ManifestId(1_111_111_111_111_111_111),
            2_000_000_000,
        );

        let written = state.to_string();
        // The other depot is untouched.
        assert!(written.contains("\"6403079453713498174\""));
        // The updated one carries its new values.
        assert!(written.contains("\"1111111111111111111\""));
        assert!(written.contains("\"2000000000\""));
        // And nothing else moved: same line count, same field order.
        assert_eq!(written.lines().count(), REAL.lines().count());
    }

    #[test]
    fn fields_this_crate_does_not_model_survive_a_rewrite() {
        // Valve adds fields. An install that lost ScheduledAutoUpdate on every
        // update would be quietly degrading a file other tools read.
        let mut state = valheim();
        state.mark_installed(22_000_000, 1_756_871_901, 1_787_752_999);

        let written = state.to_string();
        for field in [
            "ScheduledAutoUpdate",
            "AllowOtherDownloadsWhileRunning",
            "AutoUpdateBehavior",
            "MountedConfig",
        ] {
            assert!(written.contains(field), "{field} was dropped by a rewrite");
        }
        assert_eq!(state.build_id(), Some(22_000_000));
    }

    #[test]
    fn a_fresh_record_reads_back_as_what_was_written() {
        let mut state = AppState::new(AppId(232_250), "Team Fortress 2 Dedicated Server", "tf2");
        state.set_depot(DepotId(232_250), ManifestId(42), 1024);
        state.mark_installed(17_442_188, 1024, 1_700_000_000);

        let text = state.to_string();
        let reread = AppState::parse(&text, AppId(232_250)).expect("must reparse");

        assert_eq!(reread.name(), Some("Team Fortress 2 Dedicated Server"));
        assert_eq!(reread.build_id(), Some(17_442_188));
        assert_eq!(
            reread.installed_manifest(DepotId(232_250)),
            Some(ManifestId(42))
        );
        assert!(reread.is_fully_installed());
        assert_eq!(reread.to_string(), text, "a fresh record must be stable");
    }

    #[test]
    fn removing_a_depot_takes_it_out_of_the_record() {
        // An app that stops shipping a depot must not leave it listed, or the
        // next update believes content is present that is not.
        let mut state = valheim();
        state.remove_depot(DepotId(1006));

        assert_eq!(state.installed_depots().len(), 1);
        assert_eq!(state.installed_manifest(DepotId(1006)), None);
        assert!(!state.to_string().contains("6403079453713498174"));
        // The other depot survived.
        assert!(state.to_string().contains("962159520942340660"));
    }

    #[test]
    fn a_missing_record_is_not_an_error() {
        // A fresh install has none, and that is not a failure.
        let root = std::path::Path::new("/nonexistent/install/root");
        assert_eq!(AppState::read(root, AppId(1)), Ok(None));
    }

    #[test]
    fn the_record_lives_where_steam_puts_it() {
        assert_eq!(
            AppState::path_for(std::path::Path::new("/srv/valheim"), AppId(896_660)),
            std::path::Path::new("/srv/valheim/steamapps/appmanifest_896660.acf")
        );
    }
}
