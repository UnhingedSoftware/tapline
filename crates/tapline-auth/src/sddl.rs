#![cfg_attr(not(windows), allow(dead_code))]

const LOCAL_SYSTEM: &str = "S-1-5-18";
const ADMINISTRATORS: &str = "S-1-5-32-544";
const LOCAL_SYSTEM_ALIAS: &str = "SY";
const ADMINISTRATORS_ALIAS: &str = "BA";

const ALLOW_ACE_TYPES: [&str; 4] = ["A", "OA", "XA", "ZA"];

/// An access-control list granting `owner` full control and naming nobody else,
/// protected so that nothing is inherited from the directory above it.
pub fn private_dacl(owner: &str) -> String {
    format!("D:P(A;;FA;;;{owner})")
}

/// Names the first account other than the owner that this descriptor grants
/// access to, or `None` if the file is the owner's alone.
///
/// `ours` answers whether one grantee is the owner. It is a parameter rather
/// than a string comparison because Windows does not hand a SID back the way it
/// was written: a descriptor read from a file renders every well-known SID as
/// its two-letter SDDL alias, so the account that wrote `S-1-5-21-...-500` reads
/// back as `LA`. Only something that can resolve both forms to real SIDs can say
/// they are one account, and that is Windows.
pub fn shared_with(dacl: &str, ours: impl Fn(&str) -> bool) -> Option<String> {
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
        if !is_ours(holder, &ours) {
            return Some(holder.to_owned());
        }
    }
    None
}

fn is_ours(holder: &str, ours: &impl Fn(&str) -> bool) -> bool {
    ours(holder)
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

/// The access-control entries in a DACL body, each without its brackets.
///
/// Splitting on `(` and cutting at the first `)` looks like it would do, and
/// does for an ordinary ACE, but a conditional one carries a bracketed
/// expression of its own: `(XA;;FA;;;WD;(Member_of {SID}))`. The naive split
/// left the grant itself unterminated, so it was dropped, and a file shared
/// with everyone through a conditional ACE read back as private. `XA` and `ZA`
/// are in `ALLOW_ACE_TYPES`, so those are entries this is meant to see.
/// Counting brackets keeps each ACE whole.
fn aces(body: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut depth: usize = 0;
    let mut start: Option<usize> = None;
    for (at, byte) in body.bytes().enumerate() {
        match byte {
            b'(' => {
                if depth == 0 {
                    start = Some(at.saturating_add(1));
                }
                depth = depth.saturating_add(1);
            }
            b')' => {
                if depth == 0 {
                    continue;
                }
                depth -= 1;
                if depth == 0
                    && let Some(from) = start.take()
                    && let Some(ace) = body.get(from..at)
                {
                    found.push(ace);
                }
            }
            _ => {}
        }
    }
    // A bracket that never closes is a descriptor we do not understand. Keeping
    // what it opened means the grantee inside it is still read and can still be
    // reported, rather than a malformed list quietly reading as private.
    if depth > 0
        && let Some(from) = start
        && let Some(ace) = body.get(from..)
    {
        found.push(ace);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: &str = "S-1-5-21-1111111111-2222222222-3333333333-1001";

    /// What the comparison used to be, before Windows' alias substitution made
    /// a real SID comparison necessary. It is still what every case below but
    /// the alias one needs.
    fn owned_by(owner: &str) -> impl Fn(&str) -> bool + '_ {
        move |holder| holder.eq_ignore_ascii_case(owner)
    }

    #[test]
    fn the_descriptor_we_write_grants_the_owner_alone() {
        let dacl = private_dacl(OWNER);
        assert_eq!(dacl, format!("D:P(A;;FA;;;{OWNER})"));
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), None);
    }

    #[test]
    fn a_descriptor_with_no_dacl_at_all_is_open_to_everyone() {
        assert_eq!(
            shared_with("O:BAG:BA", owned_by(OWNER)),
            Some("everyone".to_owned()),
            "a security descriptor with no DACL grants everyone access"
        );
    }

    #[test]
    fn an_explicitly_absent_dacl_is_open_to_everyone() {
        assert_eq!(
            shared_with("D:NO_ACCESS_CONTROL", owned_by(OWNER)),
            Some("everyone".to_owned())
        );
    }

    #[test]
    fn the_accounts_that_can_reach_any_file_anyway_do_not_count() {
        let dacl = format!("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), None);

        let spelled_out = format!("D:P(A;;FA;;;S-1-5-18)(A;;FA;;;S-1-5-32-544)(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&spelled_out, owned_by(OWNER)), None);
    }

    #[test]
    fn another_account_with_any_access_at_all_is_reported() {
        let dacl = format!("D:P(A;;FA;;;{OWNER})(A;;0x1200a9;;;BU)");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), Some("BU".to_owned()));
    }

    #[test]
    fn everyone_is_reported_even_when_the_owner_is_listed_first() {
        let dacl = format!("D:AI(A;;FA;;;{OWNER})(A;ID;FA;;;WD)");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), Some("WD".to_owned()));
    }

    #[test]
    fn a_denial_of_someone_else_is_not_a_grant_to_them() {
        let dacl = format!("D:P(D;;FA;;;WD)(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), None);
    }

    #[test]
    fn an_owner_sid_matches_whatever_case_windows_hands_back() {
        let dacl = private_dacl(&OWNER.to_lowercase());
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), None);
    }

    #[test]
    fn a_system_acl_after_the_dacl_is_not_read_as_a_grant() {
        let dacl = format!("D:P(A;;FA;;;{OWNER})S:AI(AU;SAFA;FA;;;WD)");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), None);
    }

    #[test]
    fn an_owner_and_group_prefix_does_not_hide_the_dacl() {
        let dacl = format!("O:{OWNER}G:BAD:P(A;;FA;;;{OWNER})");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), None);

        let shared = format!("O:{OWNER}G:BAD:P(A;;FA;;;{OWNER})(A;;FA;;;WD)");
        assert_eq!(shared_with(&shared, owned_by(OWNER)), Some("WD".to_owned()));
    }

    #[test]
    fn an_alias_windows_substituted_for_our_own_sid_is_still_us() {
        // The regression the Windows job caught: we write the owner's numeric
        // SID, Windows reads it back as `LA`, and comparing the text refuses
        // the owner their own file.
        let dacl = "D:P(A;;FA;;;LA)";
        assert_eq!(
            shared_with(dacl, owned_by(OWNER)),
            Some("LA".to_owned()),
            "text alone cannot tell that an alias is the owner"
        );
        assert_eq!(
            shared_with(dacl, |holder| holder == "LA"),
            None,
            "a caller that can resolve the alias is believed"
        );
    }

    #[test]
    fn an_empty_dacl_grants_nobody_anything() {
        assert_eq!(shared_with("D:P", owned_by(OWNER)), None);
    }
}

#[cfg(test)]
mod conditional_ace_tests {
    use super::{aces, shared_with};

    const OWNER: &str = "S-1-5-21-1111111111-2222222222-3333333333-1001";

    fn owned_by(owner: &str) -> impl Fn(&str) -> bool + '_ {
        move |holder| holder.eq_ignore_ascii_case(owner)
    }

    #[test]
    fn a_conditional_ace_keeps_its_own_brackets() {
        let body = "P(A;;FA;;;S-1-5-21-1)(XA;;FA;;;WD;(Member_of {SID(BA)}))";
        assert_eq!(
            aces(body),
            vec!["A;;FA;;;S-1-5-21-1", "XA;;FA;;;WD;(Member_of {SID(BA)})"]
        );
    }

    #[test]
    fn a_conditional_grant_to_everyone_is_reported() {
        // Before brackets were counted this came back `None`: the grant was cut
        // in half, dropped, and the file read as the owner's alone.
        let dacl = format!("D:P(A;;FA;;;{OWNER})(XA;;FA;;;WD;(Member_of {{SID(BA)}}))");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), Some("WD".to_owned()));
    }

    #[test]
    fn an_unterminated_ace_is_not_silently_dropped() {
        let dacl = format!("D:P(A;;FA;;;{OWNER})(A;;FA;;;WD");
        assert_eq!(shared_with(&dacl, owned_by(OWNER)), Some("WD".to_owned()));
    }
}
