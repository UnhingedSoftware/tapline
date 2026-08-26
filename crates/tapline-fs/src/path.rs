//! Validating a manifest path before anything is opened.

use std::fmt;
use std::path::{Component, Path, PathBuf};

/// Why a path from a manifest was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The path was empty, or resolved to the install root itself.
    Empty,
    /// An absolute path, which would ignore the install directory entirely.
    Absolute,
    /// A `..` component, anywhere in the path.
    ParentTraversal,
    /// A drive letter, UNC share or other filesystem prefix.
    ///
    /// Windows-shaped, and worth rejecting even on Linux: a manifest is not
    /// platform-specific, and a path that is dangerous on one target should not
    /// be silently accepted on another.
    Prefix,
    /// A NUL byte, which truncates the path at the syscall boundary so that
    /// what was checked is not what gets opened.
    InteriorNul,
    /// A symlink whose target resolves outside the install root.
    SymlinkEscapes {
        /// The link's own path.
        link: String,
        /// Where it pointed.
        target: String,
    },
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the path is empty or names the install root"),
            Self::Absolute => f.write_str("the path is absolute"),
            Self::ParentTraversal => f.write_str("the path contains a `..` component"),
            Self::Prefix => f.write_str("the path carries a drive or UNC prefix"),
            Self::InteriorNul => f.write_str("the path contains a NUL byte"),
            Self::SymlinkEscapes { link, target } => {
                write!(
                    f,
                    "the symlink {link} points outside the install root, at {target}"
                )
            }
        }
    }
}

impl std::error::Error for PathError {}

/// A manifest path that has been checked and is safe to join onto a root.
///
/// The only way to build one is [`validate_path`], so a function taking a
/// `SafePath` cannot be handed an unchecked one by accident. That is the point:
/// the check is not something a caller can forget to do.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SafePath(PathBuf);

impl SafePath {
    /// The relative path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// The path joined onto `root`.
    #[must_use]
    pub fn resolve(&self, root: &Path) -> PathBuf {
        root.join(&self.0)
    }

    /// The path as the manifest wrote it, with forward slashes.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string_lossy().replace('\\', "/")
    }
}

impl fmt::Display for SafePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_str())
    }
}

/// Checks a path from a manifest.
///
/// Rejects on any doubt. This is a pure function of the path text — it never
/// touches the filesystem, because a question answered against the disk can have
/// a different answer a moment later, and the gap between the check and the open
/// is exactly where that matters.
pub fn validate_path(raw: &str) -> Result<SafePath, PathError> {
    if raw.contains('\0') {
        return Err(PathError::InteriorNul);
    }

    // Manifests use backslashes even for Linux depots, so both separators are
    // treated as separators — otherwise `..\..\etc` would look like one long
    // filename and sail through the `..` check.
    let normalised = raw.replace('\\', "/");
    if normalised.is_empty() {
        return Err(PathError::Empty);
    }

    let path = Path::new(&normalised);
    let mut cleaned = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => cleaned.push(part),
            // `./a` is harmless and Valve does emit it.
            Component::CurDir => {}
            Component::ParentDir => return Err(PathError::ParentTraversal),
            Component::RootDir => return Err(PathError::Absolute),
            Component::Prefix(_) => return Err(PathError::Prefix),
        }
    }

    if cleaned.as_os_str().is_empty() {
        return Err(PathError::Empty);
    }
    Ok(SafePath(cleaned))
}

/// Checks a symlink's target, resolved against the link's own directory.
///
/// A symlink is the indirect form of a traversal: the link itself sits happily
/// inside the root while its target does not, and a later write through the link
/// lands wherever the target points. Both a relative target that climbs out and
/// an absolute one are refused.
pub fn validate_symlink(link: &SafePath, target: &str) -> Result<PathBuf, PathError> {
    if target.contains('\0') {
        return Err(PathError::InteriorNul);
    }
    let normalised = target.replace('\\', "/");
    if normalised.is_empty() {
        return Err(PathError::Empty);
    }

    let target_path = Path::new(&normalised);
    if target_path.is_absolute() {
        return Err(PathError::SymlinkEscapes {
            link: link.as_str(),
            target: normalised,
        });
    }

    // Walk the link's directory plus the target, tracking depth. Depth going
    // negative at any point means the path left the root, whatever it does
    // afterwards — `../../a/b/c` is an escape even though it ends up three
    // levels deep again.
    let mut depth: i64 = link
        .as_path()
        .parent()
        .map(|parent| parent.components().filter(is_normal).count() as i64)
        .unwrap_or(0);

    for component in target_path.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(PathError::SymlinkEscapes {
                        link: link.as_str(),
                        target: normalised,
                    });
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(PathError::SymlinkEscapes {
                    link: link.as_str(),
                    target: normalised,
                });
            }
        }
    }

    Ok(target_path.to_path_buf())
}

/// Whether a component names a real path segment.
fn is_normal(component: &Component<'_>) -> bool {
    matches!(component, Component::Normal(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_depot_paths_are_accepted() {
        // What a real manifest actually contains.
        for path in [
            "tf/cfg/pure_server_whitelist.txt",
            "bin/linux64/srcds_linux",
            "tf/cfg/unencrypted/print_instance_config.py",
            "single-file",
            "./with/a/leading/dot",
        ] {
            validate_path(path).unwrap_or_else(|e| panic!("{path} was refused: {e}"));
        }
    }

    #[test]
    fn windows_separators_are_treated_as_separators() {
        // Valve writes backslashes even in Linux depots. Treating them as part
        // of a filename would let `..\..\etc\passwd` through as one long name.
        let path = validate_path("bin\\linux64\\srcds").expect("must accept");
        assert_eq!(path.as_str(), "bin/linux64/srcds");

        assert_eq!(
            validate_path("..\\..\\etc\\passwd"),
            Err(PathError::ParentTraversal)
        );
    }

    #[test]
    fn absolute_paths_are_refused() {
        assert_eq!(
            validate_path("/etc/cron.d/payload"),
            Err(PathError::Absolute)
        );
        assert_eq!(validate_path("/"), Err(PathError::Absolute));
    }

    #[test]
    fn parent_traversal_is_refused_wherever_it_appears() {
        // Not just a leading `../`. A `..` in the middle normalises out of the
        // root just as effectively.
        for path in [
            "../escape",
            "../../../../etc/passwd",
            "a/../../b",
            "deeply/nested/path/../../../../../../tmp/x",
            "a/..",
        ] {
            assert_eq!(
                validate_path(path),
                Err(PathError::ParentTraversal),
                "{path} was not refused"
            );
        }
    }

    #[test]
    fn empty_and_dot_only_paths_are_refused() {
        // These resolve to the install root itself, turning a file write into a
        // clobber of the directory.
        assert_eq!(validate_path(""), Err(PathError::Empty));
        assert_eq!(validate_path("."), Err(PathError::Empty));
        assert_eq!(validate_path("./"), Err(PathError::Empty));
        assert_eq!(validate_path("././."), Err(PathError::Empty));
    }

    #[test]
    fn a_nul_byte_is_refused() {
        // A NUL truncates the path at the syscall boundary, so what was checked
        // is not what gets opened.
        assert_eq!(
            validate_path("safe.txt\0/../../etc/passwd"),
            Err(PathError::InteriorNul)
        );
    }

    #[test]
    fn a_name_that_merely_contains_dots_is_fine() {
        // `..` is only dangerous as a whole component. Refusing every path with
        // two dots in it would reject real filenames.
        validate_path("libstdc++.so.6").expect("must accept");
        validate_path("weird..name.txt").expect("must accept");
        validate_path("a/..b/c").expect("must accept");
    }

    #[test]
    fn a_relative_symlink_inside_the_root_is_accepted() {
        let link = validate_path("bin/linux64/libsteam.so").expect("valid link path");
        validate_symlink(&link, "../libsteam_api.so").expect("must accept");
        validate_symlink(&link, "libsteam_api.so").expect("must accept");
        // Down and back up again, never leaving the root.
        validate_symlink(&link, "../../bin/other.so").expect("must accept");
    }

    #[test]
    fn a_symlink_climbing_out_of_the_root_is_refused() {
        // The link sits happily inside the root; its target does not, and a
        // write through it lands wherever the target points.
        let link = validate_path("bin/evil").expect("valid link path");
        assert!(matches!(
            validate_symlink(&link, "../../../../etc/passwd"),
            Err(PathError::SymlinkEscapes { .. })
        ));
    }

    #[test]
    fn a_symlink_that_escapes_and_returns_is_still_refused() {
        // `../../a/b` ends up inside a directory again, but it left the root on
        // the way — and the intermediate path is what an attacker controls.
        let link = validate_path("a/link").expect("valid link path");
        assert!(matches!(
            validate_symlink(&link, "../../elsewhere/b"),
            Err(PathError::SymlinkEscapes { .. })
        ));
    }

    #[test]
    fn an_absolute_symlink_target_is_refused() {
        let link = validate_path("bin/evil").expect("valid link path");
        assert!(matches!(
            validate_symlink(&link, "/etc/passwd"),
            Err(PathError::SymlinkEscapes { .. })
        ));
        assert!(matches!(
            validate_symlink(&link, "\\windows\\system32"),
            Err(PathError::SymlinkEscapes { .. })
        ));
    }

    #[test]
    fn a_safe_path_resolves_under_the_root_it_is_given() {
        let path = validate_path("tf/cfg/server.cfg").expect("must accept");
        let resolved = path.resolve(Path::new("/srv/tf2"));
        assert_eq!(resolved, Path::new("/srv/tf2/tf/cfg/server.cfg"));
        assert!(
            resolved.starts_with("/srv/tf2"),
            "a validated path resolved outside its root"
        );
    }

    #[test]
    fn every_accepted_path_resolves_under_its_root() {
        // The property the whole module exists for, stated once directly.
        let root = Path::new("/srv/install");
        for candidate in [
            "a",
            "a/b/c",
            "./a/b",
            "a\\b",
            "libstdc++.so.6",
            "a/..b",
            "x/y/z.bin",
        ] {
            if let Ok(path) = validate_path(candidate) {
                let resolved = path.resolve(root);
                assert!(
                    resolved.starts_with(root),
                    "{candidate} resolved to {resolved:?}, outside {root:?}"
                );
                assert!(
                    !resolved
                        .components()
                        .any(|c| matches!(c, Component::ParentDir)),
                    "{candidate} kept a parent component"
                );
            }
        }
    }
}
