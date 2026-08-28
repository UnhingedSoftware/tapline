use std::fmt;
use std::path::PathBuf;
use tapline_cdn::{CdnError, PoolError};
use tapline_fs::PathError;
use tapline_ids::{AppId, DepotId};
use tapline_manifest::ManifestError;
use tapline_net::NetError;
use tapline_pics::{DepotFilter, Os, PicsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOptions {
    pub install_dir: PathBuf,
    pub os: Os,
    pub branch: String,
    pub include_dlc: bool,
    pub resume: bool,
    pub force: bool,
    pub concurrency: usize,
    pub file_modes: FileModes,
    pub workshop_layout: WorkshopLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkshopLayout {
    #[default]
    SteamCmd,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileModes {
    #[default]
    SteamCmd,
    Manifest,
}

impl FileModes {
    #[must_use]
    pub const fn mode_for(self, executable: bool) -> u32 {
        match self {
            Self::SteamCmd => 0o755,
            Self::Manifest if executable => 0o755,
            Self::Manifest => 0o644,
        }
    }
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_dir: PathBuf::from("."),
            os: Os::host(),
            branch: "public".to_owned(),
            include_dlc: false,
            resume: true,
            force: false,
            concurrency: 48,
            file_modes: FileModes::default(),
            workshop_layout: WorkshopLayout::default(),
        }
    }
}

impl InstallOptions {
    #[must_use]
    pub fn filter(&self) -> DepotFilter {
        DepotFilter {
            os: self.os,
            branch: self.branch.clone(),
            include_dlc: self.include_dlc,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallReport {
    pub app: AppId,
    pub depots: Vec<DepotId>,
    pub files: u64,
    pub bytes_written: u64,
    pub bytes_downloaded: u64,
    pub chunks_reused: u64,
    pub depots_unchanged: u64,
    pub skipped: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    Net(NetError),
    Pics(PicsError),
    Manifest(ManifestError),
    Cdn(CdnError),
    Pool(PoolError),
    UnsafePath { path: String, reason: PathError },
    Io(String),
    NoDepotKey { depot: DepotId, eresult: i32 },
    NothingToInstall { app: AppId, branch: String },
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Net(e) => write!(f, "{e}"),
            Self::Pics(e) => write!(f, "{e}"),
            Self::Manifest(e) => write!(f, "{e}"),
            Self::Cdn(e) => write!(f, "{e}"),
            Self::Pool(e) => write!(f, "{e}"),
            Self::UnsafePath { path, reason } => {
                write!(f, "the manifest named an unsafe path {path:?}: {reason}")
            }
            Self::Io(message) => write!(f, "filesystem error: {message}"),
            Self::NoDepotKey { depot, eresult } => {
                if *eresult == ACCESS_DENIED {
                    write!(
                        f,
                        "Steam refused a key for depot {depot}: access denied. \
                         This depot is not anonymously accessible, so it needs an \
                         account that owns it — run `tapline login`, or build the \
                         session with Session::with_token"
                    )
                } else {
                    write!(
                        f,
                        "Steam granted no key for depot {depot} (EResult {eresult})"
                    )
                }
            }
            Self::NothingToInstall { app, branch } => write!(
                f,
                "app {app} has nothing to install on branch {branch} for this platform"
            ),
        }
    }
}

pub const ACCESS_DENIED: i32 = 15;

impl InstallError {
    #[must_use]
    pub fn needs_login(&self) -> bool {
        match self {
            Self::NoDepotKey { eresult, .. } => *eresult == ACCESS_DENIED,
            Self::Net(NetError::Steam { eresult }) => *eresult == ACCESS_DENIED,
            _ => false,
        }
    }
}

impl std::error::Error for InstallError {}

impl From<NetError> for InstallError {
    fn from(error: NetError) -> Self {
        Self::Net(error)
    }
}
impl From<PicsError> for InstallError {
    fn from(error: PicsError) -> Self {
        Self::Pics(error)
    }
}
impl From<ManifestError> for InstallError {
    fn from(error: ManifestError) -> Self {
        Self::Manifest(error)
    }
}
impl From<CdnError> for InstallError {
    fn from(error: CdnError) -> Self {
        Self::Cdn(error)
    }
}
impl From<PoolError> for InstallError {
    fn from(error: PoolError) -> Self {
        Self::Pool(error)
    }
}
impl From<std::io::Error> for InstallError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_options_install_for_this_platform_from_public() {
        let options = InstallOptions::default();
        assert_eq!(options.branch, "public");
        assert!(!options.include_dlc);
        assert_eq!(options.os, Os::host());
    }

    #[test]
    fn the_filter_reflects_the_options() {
        let options = InstallOptions {
            os: Os::Windows,
            branch: "beta".to_owned(),
            include_dlc: true,
            ..InstallOptions::default()
        };
        let filter = options.filter();
        assert_eq!(filter.os, Os::Windows);
        assert_eq!(filter.branch, "beta");
        assert!(filter.include_dlc);
    }

    #[test]
    fn concurrency_defaults_to_something_a_cdn_will_tolerate() {
        let concurrency = InstallOptions::default().concurrency;
        assert!(
            (1..=64).contains(&concurrency),
            "default concurrency {concurrency} is outside the measured range"
        );
    }

    #[test]
    fn the_default_concurrency_is_the_measured_plateau() {
        const PLATEAU: usize = 48;

        let concurrency = InstallOptions::default().concurrency;
        assert_eq!(
            concurrency, PLATEAU,
            "the default should sit at the measured plateau; \
             above it is slower and dearer, below it is slower and cheaper"
        );
    }

    #[test]
    fn an_access_denied_depot_says_to_sign_in() {
        let error = InstallError::NoDepotKey {
            depot: DepotId(4001),
            eresult: ACCESS_DENIED,
        };
        let text = error.to_string();
        assert!(text.contains("access denied"), "{text}");
        assert!(text.contains("tapline login"), "{text}");
        assert!(error.needs_login());
    }

    #[test]
    fn another_refusal_is_not_reported_as_a_login_problem() {
        let error = InstallError::NoDepotKey {
            depot: DepotId(4001),
            eresult: 2,
        };
        assert!(!error.needs_login());
        assert!(!error.to_string().contains("tapline login"));
    }

    #[test]
    fn an_unsafe_path_reads_as_a_refusal_rather_than_a_skip() {
        let error = InstallError::UnsafePath {
            path: "../../etc/passwd".to_owned(),
            reason: PathError::ParentTraversal,
        };
        let rendered = error.to_string();
        assert!(rendered.contains("unsafe path"), "{rendered}");
        assert!(rendered.contains("../../etc/passwd"), "{rendered}");
    }
}
