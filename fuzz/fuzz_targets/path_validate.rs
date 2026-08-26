//! The property the install path validator exists for, stated as an invariant:
//!
//! **anything it accepts must resolve inside the root.**
//!
//! Hand-written cases cover the traversals I thought of. This covers the ones I
//! did not — and it is worth fuzzing rather than merely testing, because the
//! input is attacker-authored: a Workshop item's manifest names the paths
//! tapline will create, and anyone can publish a Workshop item.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::{Component, Path};
use tapline_fs::{validate_path, validate_symlink};

/// A root that is easy to check membership of.
const ROOT: &str = "/srv/install";

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    let Ok(path) = validate_path(text) else {
        // Refusing input is the expected outcome for most of it.
        return;
    };

    let root = Path::new(ROOT);
    let resolved = path.resolve(root);

    // The invariant.
    assert!(
        resolved.starts_with(root),
        "accepted {text:?} which resolved to {resolved:?}, outside {root:?}"
    );

    // And no component that could still climb out once the OS resolves it.
    for component in resolved.components() {
        assert!(
            !matches!(component, Component::ParentDir),
            "accepted {text:?} keeping a parent component: {resolved:?}"
        );
    }

    // A validated path must never be empty, or it names the root itself.
    assert!(
        !path.as_path().as_os_str().is_empty(),
        "accepted {text:?} as an empty path"
    );

    // Nor carry a NUL, which would truncate at the syscall boundary and make
    // what was checked differ from what is opened.
    assert!(
        !path.as_str().contains('\0'),
        "accepted {text:?} containing a NUL"
    );

    // Symlink targets get the same treatment, resolved against the link's own
    // directory.
    if let Ok(target) = validate_symlink(&path, text) {
        let link_dir = path.as_path().parent().unwrap_or(Path::new(""));
        let combined = root.join(link_dir).join(&target);

        // Resolve `..` textually the way the kernel would, then check the
        // result is still inside.
        let mut depth = 0_i64;
        let mut escaped = false;
        for component in combined.strip_prefix(root).unwrap_or(&combined).components() {
            match component {
                Component::Normal(_) => depth += 1,
                Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        escaped = true;
                    }
                }
                _ => {}
            }
        }
        assert!(
            !escaped,
            "accepted symlink {text:?} -> {target:?} which escapes {root:?}"
        );
    }
});
