use crate::{InstallError, InstallOptions};
use std::fmt;
use tapline_ids::{AppId, DepotId, ManifestId, PublishedFileId};

const RESULT_OK: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkshopContent {
    SteamPipe {
        depot: DepotId,
        manifest: ManifestId,
    },
    Legacy {
        url: String,
        filename: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkshopItem {
    pub id: PublishedFileId,
    pub app: AppId,
    pub title: String,
    pub size: u64,
    pub updated: u32,
    pub content: WorkshopContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkshopError {
    Refused { id: PublishedFileId, eresult: u32 },
    NoContent { id: PublishedFileId },
    NoWorkshopDepot { app: AppId },
    NotReturned { id: PublishedFileId },
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

#[must_use]
pub fn item_dir(root: &std::path::Path, app: AppId, id: PublishedFileId) -> std::path::PathBuf {
    root.join("steamapps")
        .join("workshop")
        .join("content")
        .join(app.to_string())
        .join(id.to_string())
}

#[must_use]
pub fn options_for(base: &InstallOptions, app: AppId, id: PublishedFileId) -> InstallOptions {
    InstallOptions {
        install_dir: target_dir(base, app, id),
        ..base.clone()
    }
}

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
        let item = classify(
            &details(1, 1, Some(0), Some("https://example.invalid/blob")),
            Some(DepotId(4000)),
        )
        .expect("must classify");
        assert!(matches!(item.content, WorkshopContent::Legacy { .. }));
    }

    #[test]
    fn a_refused_item_carries_steams_own_code() {
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
        assert_eq!(
            InstallOptions::default().workshop_layout,
            crate::WorkshopLayout::SteamCmd
        );
    }
}
