//! Workshop items.
//!
//! A published file reaches a client one of two ways, and
//! `PublishedFile.GetDetails` says which:
//!
//! * **SteamPipe UGC** — `hcontent_file` is a manifest id inside the app's
//!   workshop depot. The item then downloads exactly like depot content, which
//!   means everything M5 and M6 built applies unchanged: request code, manifest,
//!   chunks, decrypt, decompress, verify.
//! * **Legacy UFS** — `file_url` is a plain HTTPS blob with no chunking and no
//!   encryption.
//!
//! Measured on 2026-08-26 across twelve real items from Garry's Mod, Arma 3 and
//! Counter-Strike 2: **eight SteamPipe, zero legacy**. The legacy path is
//! implemented anyway because it is twenty lines and a `file_url` does appear in
//! the wild — a Steam screenshot carries both — but the SteamPipe path is the
//! one that matters.
//!
//! # Workshop content is the hostile case
//!
//! Anyone can publish a Workshop item, and its manifest names the paths tapline
//! will create. This is the input `tapline-fs` exists for, and it is why an
//! unsafe path here is a fatal error rather than a skipped file.

use crate::{InstallError, InstallOptions};
use std::fmt;
use tapline_ids::{AppId, DepotId, ManifestId, PublishedFileId};

/// Steam's result code for success, on a per-item basis.
const RESULT_OK: u32 = 1;

/// Where an item's content lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkshopContent {
    /// Content in the app's workshop depot, addressed by manifest.
    ///
    /// The overwhelmingly common case.
    SteamPipe {
        /// The depot the app keeps Workshop content in.
        depot: DepotId,
        /// The item's manifest, from `hcontent_file`.
        manifest: ManifestId,
    },
    /// A plain HTTPS blob.
    Legacy {
        /// Where to fetch it.
        url: String,
        /// The name to save it under, when the item gives one.
        filename: Option<String>,
    },
}

/// A Workshop item, as Steam describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkshopItem {
    /// The published file id.
    pub id: PublishedFileId,
    /// The app the item belongs to.
    pub app: AppId,
    /// Its title, which may be empty.
    pub title: String,
    /// Its size in bytes, as Steam reports it.
    pub size: u64,
    /// When it was last updated, as a Unix timestamp.
    pub updated: u32,
    /// How to get it.
    pub content: WorkshopContent,
}

/// Why an item could not be described or downloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkshopError {
    /// Steam returned a non-success result for the item.
    ///
    /// Carries the code, because the difference between "no such item" and
    /// "you may not see it" is the difference between a typo and a permission
    /// problem.
    Refused {
        /// Which item.
        id: PublishedFileId,
        /// Steam's `EResult`.
        eresult: u32,
    },
    /// Steam described the item but gave no way to fetch it.
    ///
    /// Neither a manifest nor a URL. Four of twelve real items came back this
    /// way — an item can exist and still have nothing an anonymous session may
    /// download.
    NoContent {
        /// Which item.
        id: PublishedFileId,
    },
    /// The item's app publishes no workshop depot.
    ///
    /// Without one a SteamPipe item has no depot to fetch a manifest from.
    NoWorkshopDepot {
        /// The app.
        app: AppId,
    },
    /// Steam did not return the item at all.
    NotReturned {
        /// Which item.
        id: PublishedFileId,
    },
}

impl fmt::Display for WorkshopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused { id, eresult } => {
                write!(f, "Steam refused Workshop item {id} (EResult {eresult})")
            }
            Self::NoContent { id } => write!(
                f,
                "Workshop item {id} has neither a manifest nor a download URL"
            ),
            Self::NoWorkshopDepot { app } => {
                write!(f, "app {app} publishes no workshop depot")
            }
            Self::NotReturned { id } => write!(f, "Steam did not return Workshop item {id}"),
        }
    }
}

impl std::error::Error for WorkshopError {}

/// Turns one `PublishedFileDetails` into an item, given the app's workshop
/// depot.
///
/// Split out so the classification — which is the part with rules — is testable
/// without a network.
pub fn classify(
    details: &tapline_proto::steammessages_publishedfile_steamclient::PublishedFileDetails,
    workshop_depot: Option<DepotId>,
) -> Result<WorkshopItem, WorkshopError> {
    let id = PublishedFileId(details.publishedfileid.unwrap_or(0));

    let result = details.result.unwrap_or(0);
    if result != RESULT_OK {
        return Err(WorkshopError::Refused {
            id,
            eresult: result,
        });
    }

    let app = AppId(details.consumer_appid.unwrap_or(0));

    // A manifest wins over a URL. An item can carry both — a Steam screenshot
    // does — and the manifest is the real content while the URL is the image
    // CDN's copy.
    let manifest = details.hcontent_file.filter(|handle| *handle != 0);
    let url = details
        .file_url
        .as_deref()
        .filter(|url| !url.is_empty())
        .map(str::to_owned);

    let content = match (manifest, url) {
        (Some(handle), _) => {
            let depot = workshop_depot.ok_or(WorkshopError::NoWorkshopDepot { app })?;
            WorkshopContent::SteamPipe {
                depot,
                manifest: ManifestId(handle),
            }
        }
        (None, Some(url)) => WorkshopContent::Legacy {
            url,
            filename: details
                .filename
                .as_deref()
                .filter(|name| !name.is_empty())
                .map(str::to_owned),
        },
        (None, None) => return Err(WorkshopError::NoContent { id }),
    };

    Ok(WorkshopItem {
        id,
        app,
        title: details.title.clone().unwrap_or_default(),
        size: details.file_size.unwrap_or(0),
        updated: details.time_updated.unwrap_or(0),
        content,
    })
}

/// Where an item's content is installed under a root.
///
/// Matches the layout the Steam client uses, so a server configured to load
/// Workshop content from a steamcmd install finds it in the same place.
#[must_use]
pub fn item_dir(root: &std::path::Path, app: AppId, id: PublishedFileId) -> std::path::PathBuf {
    root.join("steamapps")
        .join("workshop")
        .join("content")
        .join(app.to_string())
        .join(id.to_string())
}

/// Options for a Workshop download, derived from install options.
#[must_use]
pub fn options_for(base: &InstallOptions, app: AppId, id: PublishedFileId) -> InstallOptions {
    InstallOptions {
        install_dir: target_dir(base, app, id),
        ..base.clone()
    }
}

/// Where this item's files will actually be written.
#[must_use]
pub fn target_dir(base: &InstallOptions, app: AppId, id: PublishedFileId) -> std::path::PathBuf {
    match base.workshop_layout {
        crate::WorkshopLayout::SteamCmd => item_dir(&base.install_dir, app, id),
        crate::WorkshopLayout::Flat => base.install_dir.clone(),
    }
}

impl From<WorkshopError> for InstallError {
    fn from(error: WorkshopError) -> Self {
        Self::Io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_proto::steammessages_publishedfile_steamclient::PublishedFileDetails;

    fn details(
        id: u64,
        result: u32,
        hcontent: Option<u64>,
        url: Option<&str>,
    ) -> PublishedFileDetails {
        PublishedFileDetails {
            publishedfileid: Some(id),
            result: Some(result),
            consumer_appid: Some(4000),
            title: Some("An Addon".to_owned()),
            file_size: Some(5194),
            time_updated: Some(1_700_000_000),
            hcontent_file: hcontent,
            file_url: url.map(str::to_owned),
            filename: Some(String::new()),
            ..PublishedFileDetails::default()
        }
    }

    #[test]
    fn a_steampipe_item_resolves_to_a_manifest_in_the_workshop_depot() {
        // The path eight of twelve real items took.
        let item = classify(
            &details(3_790_437_566, 1, Some(2_351_397_028_775_917_616), None),
            Some(DepotId(4000)),
        )
        .expect("must classify");

        assert_eq!(item.id, PublishedFileId(3_790_437_566));
        assert_eq!(item.app, AppId(4000));
        assert_eq!(item.size, 5194);
        assert_eq!(
            item.content,
            WorkshopContent::SteamPipe {
                depot: DepotId(4000),
                manifest: ManifestId(2_351_397_028_775_917_616),
            }
        );
    }

    #[test]
    fn a_manifest_wins_over_a_url_when_an_item_has_both() {
        // A Steam screenshot carries both: hcontent_file is the real content and
        // file_url is the image CDN's copy. Taking the URL would download a
        // JPEG where the item's actual content was asked for.
        let item = classify(
            &details(
                2_942_526_891,
                1,
                Some(2_010_332_231_404_597_398),
                Some("https://images.steamusercontent.com/ugc/2010332231404597398/ABC/"),
            ),
            Some(DepotId(4000)),
        )
        .expect("must classify");

        assert!(
            matches!(item.content, WorkshopContent::SteamPipe { .. }),
            "the URL was preferred over the manifest"
        );
    }

    #[test]
    fn a_legacy_item_falls_back_to_its_url() {
        let item = classify(
            &details(1, 1, None, Some("https://example.invalid/blob")),
            Some(DepotId(4000)),
        )
        .expect("must classify");

        assert_eq!(
            item.content,
            WorkshopContent::Legacy {
                url: "https://example.invalid/blob".to_owned(),
                filename: None,
            }
        );
    }

    #[test]
    fn a_zero_manifest_handle_is_not_a_manifest() {
        // Steam sets the field to zero rather than omitting it, and treating
        // that as a manifest id would ask the CDN for manifest 0.
        let item = classify(
            &details(1, 1, Some(0), Some("https://example.invalid/blob")),
            Some(DepotId(4000)),
        )
        .expect("must classify");
        assert!(matches!(item.content, WorkshopContent::Legacy { .. }));
    }

    #[test]
    fn a_refused_item_carries_steams_own_code() {
        // 9 is FileNotFound and 15 is AccessDenied — a typo and a permission
        // problem, and an operator needs to tell them apart.
        for eresult in [9, 15] {
            assert_eq!(
                classify(
                    &details(104_054_805, eresult, None, None),
                    Some(DepotId(4000))
                ),
                Err(WorkshopError::Refused {
                    id: PublishedFileId(104_054_805),
                    eresult,
                })
            );
        }
    }

    #[test]
    fn an_item_with_no_content_is_reported_rather_than_downloaded_as_nothing() {
        // Four of twelve real items came back this way: they exist, and there is
        // nothing an anonymous session may fetch.
        assert_eq!(
            classify(&details(1, 1, None, None), Some(DepotId(4000))),
            Err(WorkshopError::NoContent {
                id: PublishedFileId(1)
            })
        );
    }

    #[test]
    fn an_app_with_no_workshop_depot_cannot_serve_a_steampipe_item() {
        assert_eq!(
            classify(&details(1, 1, Some(42), None), None),
            Err(WorkshopError::NoWorkshopDepot { app: AppId(4000) })
        );
    }

    #[test]
    fn items_install_where_the_steam_client_puts_them() {
        // A server configured against a steamcmd install must find content in
        // the same place.
        assert_eq!(
            item_dir(
                std::path::Path::new("/srv/gmod"),
                AppId(4000),
                PublishedFileId(3_790_437_566)
            ),
            std::path::Path::new("/srv/gmod/steamapps/workshop/content/4000/3790437566")
        );
    }

    #[test]
    fn the_steamcmd_layout_builds_the_path_steamcmd_builds() {
        let options = InstallOptions {
            install_dir: std::path::PathBuf::from("/srv/gmod"),
            ..InstallOptions::default()
        };
        assert_eq!(
            target_dir(&options, AppId(4000), PublishedFileId(104_691_717)),
            std::path::PathBuf::from("/srv/gmod/steamapps/workshop/content/4000/104691717")
        );
    }

    #[test]
    fn the_flat_layout_writes_into_the_directory_it_was_given() {
        // The Garry's Mod case: garrysmod/addons is already the right folder,
        // and the steamcmd layout would put the .gma four directories below
        // where the server looks for it.
        let options = InstallOptions {
            install_dir: std::path::PathBuf::from("/srv/gmod/garrysmod/addons"),
            workshop_layout: crate::WorkshopLayout::Flat,
            ..InstallOptions::default()
        };
        assert_eq!(
            target_dir(&options, AppId(4000), PublishedFileId(104_691_717)),
            std::path::PathBuf::from("/srv/gmod/garrysmod/addons")
        );
    }

    #[test]
    fn flat_puts_two_items_side_by_side() {
        // Which is the point: a collection downloaded flat is a folder of
        // addons, not a folder of folders.
        let options = InstallOptions {
            install_dir: std::path::PathBuf::from("/srv/addons"),
            workshop_layout: crate::WorkshopLayout::Flat,
            ..InstallOptions::default()
        };
        let first = target_dir(&options, AppId(4000), PublishedFileId(1));
        let second = target_dir(&options, AppId(4000), PublishedFileId(2));
        assert_eq!(first, second);
    }

    #[test]
    fn the_default_layout_is_steamcmds() {
        // Changing this silently would move every existing consumer's files.
        assert_eq!(
            InstallOptions::default().workshop_layout,
            crate::WorkshopLayout::SteamCmd
        );
    }
}
