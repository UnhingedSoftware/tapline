use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalAccount {
    pub steam_id: u64,
    pub account: String,
    pub persona: String,
    pub most_recent: bool,
}

fn roots(home: &Path) -> [PathBuf; 3] {
    [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".steam/root"),
    ]
}

#[must_use]
pub fn discover() -> Vec<LocalAccount> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    discover_in(&roots(&home))
}

#[must_use]
pub fn discover_in(roots: &[PathBuf]) -> Vec<LocalAccount> {
    let mut seen = Vec::new();
    let mut found = Vec::new();

    for root in roots {
        let path = root.join("config/loginusers.vdf");
        let Ok(real) = std::fs::canonicalize(&path) else {
            continue;
        };
        if seen.contains(&real) {
            continue;
        }
        seen.push(real);

        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        found.extend(parse_login_users(&text));
    }

    sort_most_recent_first(&mut found);
    found
}

#[must_use]
pub fn most_recent() -> Option<LocalAccount> {
    discover().into_iter().next()
}

#[must_use]
pub fn parse_login_users(text: &str) -> Vec<LocalAccount> {
    let Ok(root) = tapline_vdf::parse(text) else {
        return Vec::new();
    };
    let Some(users) = root
        .get_object("users")
        .or_else(|| root.iter().find_map(|(_, value)| value.as_object()))
    else {
        return Vec::new();
    };

    let mut accounts: Vec<LocalAccount> = users
        .iter()
        .filter_map(|(id, value)| {
            let entry = value.as_object()?;
            let account = entry.get_str("AccountName")?;
            if account.is_empty() {
                return None;
            }
            Some(LocalAccount {
                steam_id: id.parse().unwrap_or(0),
                account: account.to_owned(),
                persona: entry.get_str("PersonaName").unwrap_or_default().to_owned(),
                most_recent: entry.get_str("MostRecent").is_some_and(|flag| flag == "1"),
            })
        })
        .collect();

    sort_most_recent_first(&mut accounts);
    accounts
}

fn sort_most_recent_first(accounts: &mut [LocalAccount]) {
    accounts.sort_by(|a, b| {
        b.most_recent
            .cmp(&a.most_recent)
            .then_with(|| a.account.cmp(&b.account))
    });
}

#[must_use]
pub fn libraries() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };

    for root in roots(&home) {
        let path = root.join("config/libraryfolders.vdf");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let found = parse_libraries(&text);
        if !found.is_empty() {
            return found;
        }
    }
    Vec::new()
}

#[must_use]
pub fn parse_libraries(text: &str) -> Vec<PathBuf> {
    let Ok(root) = tapline_vdf::parse(text) else {
        return Vec::new();
    };
    let Some(folders) = root
        .get_object("libraryfolders")
        .or_else(|| root.iter().find_map(|(_, value)| value.as_object()))
    else {
        return Vec::new();
    };

    folders
        .iter()
        .filter_map(|(_, value)| match value.as_object() {
            Some(entry) => entry.get_str("path").map(PathBuf::from),
            None => value.as_str().map(PathBuf::from),
        })
        .filter(|path| path.as_os_str().len() > 1)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_ACCOUNTS: &str = r#"
"users"
{
	"76561198000000001"
	{
		"AccountName"		"first"
		"PersonaName"		"The First"
		"MostRecent"		"0"
		"Timestamp"		"1700000000"
	}
	"76561198000000002"
	{
		"AccountName"		"second"
		"PersonaName"		"The Second"
		"MostRecent"		"1"
		"Timestamp"		"1800000000"
	}
}
"#;

    #[test]
    fn the_most_recent_account_comes_first() {
        let accounts = parse_login_users(TWO_ACCOUNTS);
        assert_eq!(accounts.len(), 2);
        let first = accounts.first().expect("one");
        assert_eq!(first.account, "second");
        assert!(first.most_recent);
    }

    #[test]
    fn an_account_carries_the_id_and_the_name_a_person_recognises() {
        let accounts = parse_login_users(TWO_ACCOUNTS);
        let recent = accounts.first().expect("one");
        assert_eq!(recent.steam_id, 76_561_198_000_000_002);
        assert_eq!(recent.persona, "The Second");
    }

    #[test]
    fn a_missing_file_is_no_accounts_rather_than_an_error() {
        let found = discover_in(&[PathBuf::from("/nonexistent/steam/root")]);
        assert!(found.is_empty());
    }

    #[test]
    fn rubbish_is_no_accounts_rather_than_a_panic() {
        assert!(parse_login_users("this is not vdf {{{").is_empty());
        assert!(parse_login_users("").is_empty());
    }

    #[test]
    fn an_account_with_no_name_is_skipped() {
        let text = r#"
"users"
{
	"76561198000000003"
	{
		"AccountName"		""
		"PersonaName"		"Nameless"
	}
}
"#;
        assert!(parse_login_users(text).is_empty());
    }

    #[test]
    fn a_single_account_with_no_most_recent_flag_is_still_found() {
        let text = r#"
"users"
{
	"76561198000000004"
	{
		"AccountName"		"only"
		"PersonaName"		"Only"
	}
}
"#;
        let accounts = parse_login_users(text);
        assert_eq!(accounts.len(), 1);
        assert!(!accounts.first().expect("one").most_recent);
    }

    #[test]
    fn the_same_install_reached_three_ways_is_reported_once() {
        let temp = std::env::temp_dir().join(format!("tapline-local-{}", std::process::id()));
        let real = temp.join("real");
        std::fs::create_dir_all(real.join("config")).expect("mkdir");
        std::fs::write(real.join("config/loginusers.vdf"), TWO_ACCOUNTS).expect("write");

        let link = temp.join("link");
        let _ = std::fs::remove_file(&link);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let found = discover_in(&[real.clone(), link, real.clone()]);
        assert_eq!(
            found.len(),
            2,
            "the same install was counted more than once"
        );

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    #[ignore = "reads the Steam client installed on this machine"]
    fn this_machine_reports_its_accounts_once_each() {
        let accounts = discover();
        for found in &accounts {
            println!(
                "{} ({}) steam_id={} most_recent={}",
                found.account, found.persona, found.steam_id, found.most_recent
            );
        }
        let mut names: Vec<&str> = accounts.iter().map(|a| a.account.as_str()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(
            before,
            names.len(),
            "an account was reported more than once"
        );

        for library in libraries() {
            println!("library: {}", library.display());
        }
    }

    #[test]
    fn library_paths_are_read_from_both_layouts() {
        let modern = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"/home/someone/.local/share/Steam"
		"label"		""
	}
	"1"
	{
		"path"		"/mnt/games/SteamLibrary"
	}
}
"#;
        assert_eq!(
            parse_libraries(modern),
            vec![
                PathBuf::from("/home/someone/.local/share/Steam"),
                PathBuf::from("/mnt/games/SteamLibrary"),
            ]
        );

        let ancient = r#"
"LibraryFolders"
{
	"1"		"/mnt/old/SteamLibrary"
}
"#;
        assert_eq!(
            parse_libraries(ancient),
            vec![PathBuf::from("/mnt/old/SteamLibrary")]
        );
    }
}
