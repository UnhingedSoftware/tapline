use tapline_ids::{AppId, DepotId, ManifestId};
use tapline_vdf::{Object, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    Linux,
    Windows,
    MacOs,
}

impl Os {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::MacOs => "macos",
        }
    }

    #[must_use]
    pub const fn host() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Linux
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepotFilter {
    pub os: Os,
    pub branch: String,
    pub include_dlc: bool,
}

impl Default for DepotFilter {
    fn default() -> Self {
        Self {
            os: Os::host(),
            branch: "public".to_owned(),
            include_dlc: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Depot {
    pub id: DepotId,
    pub manifest: ManifestId,
    pub size: u64,
    pub download_size: u64,
    pub owner: AppId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub build_id: Option<u64>,
    pub password_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    app_id: AppId,
    root: Object,
}

const NON_DEPOT_KEYS: &[&str] = &[
    "branches",
    "baselanguages",
    "overridescddb",
    "hasdepotsindlc",
    "privatebranches",
    "partitions",
];

impl AppInfo {
    pub fn parse(app_id: AppId, buffer: &[u8]) -> Result<Self, tapline_vdf::VdfError> {
        let trimmed = buffer.strip_suffix(&[0]).unwrap_or(buffer);
        let text = String::from_utf8_lossy(trimmed);
        let document = tapline_vdf::parse(&text)?;

        let root = document.get_object("appinfo").cloned().unwrap_or(document);

        Ok(Self { app_id, root })
    }

    #[must_use]
    pub const fn app_id(&self) -> AppId {
        self.app_id
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.root.get_object("common")?.get_str("name")
    }

    #[must_use]
    pub fn app_type(&self) -> Option<&str> {
        self.root.get_object("common")?.get_str("type")
    }

    #[must_use]
    pub const fn raw(&self) -> &Object {
        &self.root
    }

    #[must_use]
    pub fn branches(&self) -> Vec<Branch> {
        let Some(branches) = self
            .root
            .get_object("depots")
            .and_then(|d| d.get_object("branches"))
        else {
            return Vec::new();
        };

        branches
            .iter()
            .filter_map(|(name, value)| {
                let entry = value.as_object()?;
                Some(Branch {
                    name: name.to_owned(),
                    build_id: entry.get_u64("buildid"),
                    password_required: entry.get_str("pwdrequired") == Some("1"),
                })
            })
            .collect()
    }

    #[must_use]
    pub fn build_id(&self, branch: &str) -> Option<u64> {
        self.root
            .get_object("depots")?
            .get_object("branches")?
            .get_object(branch)?
            .get_u64("buildid")
    }

    #[must_use]
    pub fn depots(&self, filter: &DepotFilter) -> Vec<Depot> {
        let Some(depots) = self.root.get_object("depots") else {
            return Vec::new();
        };

        let mut out = Vec::new();
        for (key, value) in depots.iter() {
            if NON_DEPOT_KEYS.contains(&key) {
                continue;
            }
            let Ok(id) = key.parse::<u32>() else {
                continue;
            };
            let Some(entry) = value.as_object() else {
                continue;
            };

            if !self.depot_matches(entry, filter) {
                continue;
            }
            let Some(manifests) = entry.get_object("manifests") else {
                continue;
            };
            let Some(branch) = manifests.get_object(&filter.branch) else {
                continue;
            };
            let Some(gid) = branch.get_u64("gid") else {
                continue;
            };

            out.push(Depot {
                id: DepotId(id),
                manifest: ManifestId(gid),
                size: branch.get_u64("size").unwrap_or(0),
                download_size: branch.get_u64("download").unwrap_or(0),
                owner: entry
                    .get_u64("depotfromapp")
                    .and_then(|v| u32::try_from(v).ok())
                    .map_or(self.app_id, AppId),
            });
        }

        out.sort_by_key(|depot| depot.id.get());
        out
    }

    fn depot_matches(&self, entry: &Object, filter: &DepotFilter) -> bool {
        if !filter.include_dlc && entry.get("dlcappid").is_some() {
            return false;
        }

        if let Some(config) = entry.get_object("config")
            && let Some(oslist) = config.get_str("oslist")
            && !oslist.trim().is_empty()
        {
            return oslist
                .split(',')
                .any(|token| token.trim().eq_ignore_ascii_case(filter.os.token()));
        }
        true
    }

    #[must_use]
    pub fn install_size(&self, filter: &DepotFilter) -> u64 {
        self.depots(filter).iter().map(|d| d.size).sum()
    }
}

impl AppInfo {
    #[must_use]
    pub fn workshop_depot(&self) -> Option<DepotId> {
        let value = self.root.get_object("depots")?.get_u64("workshopdepot")?;
        u32::try_from(value).ok().map(DepotId)
    }
}

impl AppInfo {
    #[must_use]
    pub const fn from_object(app_id: AppId, root: Object) -> Self {
        Self { app_id, root }
    }
}

#[allow(dead_code, reason = "kept next to the accessors it mirrors")]
fn nested<'a>(object: &'a Object, path: &[&str]) -> Option<&'a Value> {
    let mut current = object;
    let (last, parents) = path.split_last()?;
    for step in parents {
        current = current.get_object(step)?;
    }
    current.get(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TF2_DS: &str = r#"
"appinfo"
{
	"appid"		"232250"
	"common"
	{
		"name"		"Team Fortress 2 Dedicated Server"
		"type"		"Tool"
		"oslist"		"windows,linux"
		"parent"		"440"
	}
	"depots"
	{
		"232250"
		{
			"manifests"
			{
				"public"
				{
					"gid"		"3447236868550150350"
					"size"		"14752673735"
					"download"		"10314244064"
				}
				"prerelease"
				{
					"gid"		"3447236868550150350"
					"size"		"14752673735"
					"download"		"10314244064"
				}
			}
		}
		"232255"
		{
			"config"
			{
				"oslist"		"windows"
			}
			"manifests"
			{
				"public"
				{
					"gid"		"428487833251189920"
					"size"		"268007501"
					"download"		"99587520"
				}
			}
		}
		"232256"
		{
			"config"
			{
				"oslist"		"linux"
			}
			"manifests"
			{
				"public"
				{
					"gid"		"698669566371320345"
					"size"		"156284679"
					"download"		"46201536"
				}
			}
		}
		"232257"
		{
			"manifests"
			{
				"public"
				{
					"gid"		"4797708003880603728"
					"size"		"9989"
					"download"		"9989"
				}
			}
		}
		"overridescddb"		"1"
		"branches"
		{
			"public"
			{
				"buildid"		"17442188"
			}
			"prerelease"
			{
				"buildid"		"17442188"
				"pwdrequired"		"1"
			}
		}
	}
}
"#;

    fn tf2() -> AppInfo {
        AppInfo::parse(AppId(232_250), TF2_DS.as_bytes()).expect("the captured response must parse")
    }

    #[test]
    fn a_nul_terminated_buffer_parses() {
        let mut buffer = TF2_DS.as_bytes().to_vec();
        buffer.push(0);
        let info = AppInfo::parse(AppId(232_250), &buffer).expect("must parse");
        assert_eq!(info.name(), Some("Team Fortress 2 Dedicated Server"));
    }

    #[test]
    fn the_app_reads_back_the_way_pics_wrote_it() {
        let info = tf2();
        assert_eq!(info.name(), Some("Team Fortress 2 Dedicated Server"));
        assert_eq!(info.app_type(), Some("Tool"));
        assert_eq!(info.build_id("public"), Some(17_442_188));
    }

    #[test]
    fn a_linux_install_leaves_the_windows_depot_behind() {
        let depots = tf2().depots(&DepotFilter {
            os: Os::Linux,
            branch: "public".to_owned(),
            include_dlc: false,
        });

        let ids: Vec<u32> = depots.iter().map(|d| d.id.get()).collect();
        assert_eq!(ids, vec![232_250, 232_256, 232_257]);
        assert!(!ids.contains(&232_255), "the Windows depot came along");
    }

    #[test]
    fn a_windows_install_leaves_the_linux_depot_behind() {
        let depots = tf2().depots(&DepotFilter {
            os: Os::Windows,
            branch: "public".to_owned(),
            include_dlc: false,
        });
        let ids: Vec<u32> = depots.iter().map(|d| d.id.get()).collect();
        assert_eq!(ids, vec![232_250, 232_255, 232_257]);
    }

    #[test]
    fn manifest_ids_and_sizes_come_through_intact() {
        let depots = tf2().depots(&DepotFilter::default());
        let shared = depots
            .iter()
            .find(|d| d.id.get() == 232_250)
            .expect("the shared depot");

        assert_eq!(shared.manifest.get(), 3_447_236_868_550_150_350);
        assert_eq!(shared.size, 14_752_673_735);
        assert_eq!(shared.download_size, 10_314_244_064);
        assert_eq!(shared.owner, AppId(232_250));
    }

    #[test]
    fn the_branches_key_is_not_mistaken_for_a_depot() {
        let depots = tf2().depots(&DepotFilter::default());
        assert!(
            depots.iter().all(|d| d.id.get() >= 232_250),
            "a non-depot key was treated as a depot"
        );
        assert_eq!(depots.len(), 3);
    }

    #[test]
    fn branches_are_listed_with_their_password_flag() {
        let branches = tf2().branches();
        let public = branches
            .iter()
            .find(|b| b.name == "public")
            .expect("a public branch");
        assert_eq!(public.build_id, Some(17_442_188));
        assert!(!public.password_required);

        let prerelease = branches
            .iter()
            .find(|b| b.name == "prerelease")
            .expect("the prerelease branch");
        assert!(prerelease.password_required);
    }

    #[test]
    fn an_unknown_branch_yields_nothing_rather_than_the_public_one() {
        let depots = tf2().depots(&DepotFilter {
            os: Os::Linux,
            branch: "no-such-branch".to_owned(),
            include_dlc: false,
        });
        assert!(depots.is_empty());
    }

    #[test]
    fn install_size_sums_the_filtered_set() {
        let filter = DepotFilter {
            os: Os::Linux,
            branch: "public".to_owned(),
            include_dlc: false,
        };
        assert_eq!(tf2().install_size(&filter), 14_908_968_403);
    }

    #[test]
    fn a_borrowed_depot_names_the_app_that_owns_it() {
        let document = r#"
            "appinfo" {
                "appid" "1"
                "depots" {
                    "9999" {
                        "depotfromapp" "228980"
                        "manifests" { "public" { "gid" "42" "size" "10" } }
                    }
                }
            }
        "#;
        let info = AppInfo::parse(AppId(1), document.as_bytes()).expect("must parse");
        let depot = info
            .depots(&DepotFilter::default())
            .into_iter()
            .next()
            .expect("one depot");
        assert_eq!(depot.owner, AppId(228_980));
    }

    #[test]
    fn dlc_depots_stay_out_unless_asked_for() {
        let document = r#"
            "appinfo" {
                "appid" "1"
                "depots" {
                    "100" { "manifests" { "public" { "gid" "1" } } }
                    "200" { "dlcappid" "555" "manifests" { "public" { "gid" "2" } } }
                }
            }
        "#;
        let info = AppInfo::parse(AppId(1), document.as_bytes()).expect("must parse");

        let without = info.depots(&DepotFilter::default());
        assert_eq!(without.len(), 1);

        let with = info.depots(&DepotFilter {
            include_dlc: true,
            ..DepotFilter::default()
        });
        assert_eq!(with.len(), 2);
    }
}
