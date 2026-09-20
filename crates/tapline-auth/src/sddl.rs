#![cfg_attr(not(windows), allow(dead_code))]

const LOCAL_SYSTEM: &str = "S-1-5-18";
const ADMINISTRATORS: &str = "S-1-5-32-544";
const LOCAL_SYSTEM_ALIAS: &str = "SY";
const ADMINISTRATORS_ALIAS: &str = "BA";

const ALLOW_ACE_TYPES: [&str; 4] = ["A", "OA", "XA", "ZA"];

pub fn private_dacl(owner: &str) -> String {
    format!("D:P(A;;FA;;;{owner})")
}

pub fn shared_with(dacl: &str, owner: &str) -> Option<String> {
    let Some(body) = dacl_body(dacl) else {
        return Some("everyone".to_owned());
    };

    for ace in aces(body) {
        let mut fields = ace.split(';');
        let Some(kind) = fields.next() else {
            continue;
        };
        if !ALLOW_ACE_TYPES.contains(&kind.trim()) {
            continue;
        }
        let Some(holder) = fields.nth(4) else {
            continue;
        };
        let holder = holder.trim();
        if !is_ours(holder, owner) {
            return Some(holder.to_owned());
        }
    }
    None
}

fn is_ours(holder: &str, owner: &str) -> bool {
    holder.eq_ignore_ascii_case(owner)
        || holder.eq_ignore_ascii_case(LOCAL_SYSTEM)
        || holder.eq_ignore_ascii_case(ADMINISTRATORS)
        || holder.eq_ignore_ascii_case(LOCAL_SYSTEM_ALIAS)
        || holder.eq_ignore_ascii_case(ADMINISTRATORS_ALIAS)
}

fn dacl_body(descriptor: &str) -> Option<&str> {
    let at = descriptor.find("D:")?;
    let body = descriptor.get(at.saturating_add(2)..)?;
    let end = body.find("S:").unwrap_or(body.len());
    let body = body.get(..end)?;
    if body.contains("NO_ACCESS_CONTROL") {
        return None;
    }
    Some(body)
}

fn aces(body: &str) -> impl Iterator<Item = &str> {
    body.split('(').skip(1).filter_map(|ace| {
        let end = ace.find(')')?;
        ace.get(..end)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "S-1-5-21-1111111111-2222222222-3333333333-1001";

    #[test]
    fn the_descriptor_we_write_grants_the_owner_alone() {
        let dacl = private_dacl(OWNER);
        assert_eq!(dacl, format!("D:P(A;;FA;;;{OWNER})"));
        assert_eq!(shared_with(&dacl, OWNER), None);
    }

    #[test]
    fn a_descriptor_with_no_dacl_at_all_is_open_to_everyone() {
        assert_eq!(
            shared_with("O:BAG:BA", OWNER),
            Some("everyone".to_owned()),
            "a security descriptor with no DACL grants everyone access"
        );
    }

    #[test]
    fn an_explicitly_absent_dacl_is_open_to_everyone() {
        assert_eq!(
            shared_with("D:NO_ACCESS_CONTROL", OWNER),
            Some("everyone".to_owned())
        );
    }

    #[test]
    fn the_accounts_that_can_reach_any_file_anyway_do_not_count() {
        let dacl = format!("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&dacl, OWNER), None);

        let spelled_out = format!("D:P(A;;FA;;;S-1-5-18)(A;;FA;;;S-1-5-32-544)(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&spelled_out, OWNER), None);
    }

    #[test]
    fn another_account_with_any_access_at_all_is_reported() {
        let dacl = format!("D:P(A;;FA;;;{OWNER})(A;;0x1200a9;;;BU)");
        assert_eq!(shared_with(&dacl, OWNER), Some("BU".to_owned()));
    }

    #[test]
    fn everyone_is_reported_even_when_the_owner_is_listed_first() {
        let dacl = format!("D:AI(A;;FA;;;{OWNER})(A;ID;FA;;;WD)");
        assert_eq!(shared_with(&dacl, OWNER), Some("WD".to_owned()));
    }

    #[test]
    fn a_denial_of_someone_else_is_not_a_grant_to_them() {
        let dacl = format!("D:P(D;;FA;;;WD)(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&dacl, OWNER), None);
    }

    #[test]
    fn an_owner_sid_matches_whatever_case_windows_hands_back() {
        let dacl = private_dacl(&OWNER.to_lowercase());
        assert_eq!(shared_with(&dacl, OWNER), None);
    }

    #[test]
    fn a_system_acl_after_the_dacl_is_not_read_as_a_grant() {
        let dacl = format!("D:P(A;;FA;;;{OWNER})S:AI(AU;SAFA;FA;;;WD)");
        assert_eq!(shared_with(&dacl, OWNER), None);
    }

    #[test]
    fn an_owner_and_group_prefix_does_not_hide_the_dacl() {
        let dacl = format!("O:{OWNER}G:BAD:P(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&dacl, OWNER), None);

        let shared = format!("O:{OWNER}G:BAD:P(A;;FA;;;{OWNER})(A;;FA;;;WD)");
        assert_eq!(shared_with(&shared, OWNER), Some("WD".to_owned()));
    }

    #[test]
    fn an_empty_dacl_grants_nobody_anything() {
        assert_eq!(shared_with("D:P", OWNER), None);
    }
}
