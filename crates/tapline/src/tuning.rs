//! Keeping the allocator from hoarding.
/// The allocator settings a download wants, as name/value pairs.
pub const ENVIRONMENT: [(&str, &str); 3] = [
    ("MALLOC_ARENA_MAX", "2"),
    ("MALLOC_TRIM_THRESHOLD_", "131072"),
    ("MALLOC_MMAP_THRESHOLD_", "131072"),
];

/// The marker that says the re-exec already happened.
pub const MARKER: &str = "TAPLINE_TUNED";

/// The opt-out.
pub const DISABLE: &str = "TAPLINE_NO_MALLOC_TUNING";

/// Re-runs the current program once with the allocator pinned; call it first in `main`.
pub fn retune() {
    // glibc only: musl has no dynamic mmap threshold and ignores these variables.
    #[cfg(all(unix, target_env = "gnu"))]
    {
        use std::os::unix::process::CommandExt;

        if !wanted(
            std::env::var_os(MARKER).is_some(),
            std::env::var_os(DISABLE).is_some(),
        ) {
            return;
        }

        let Ok(exe) = std::env::current_exe() else {
            return;
        };

        let mut command = std::process::Command::new(exe);
        command.args(std::env::args_os().skip(1)).env(MARKER, "1");
        for (name, value) in ENVIRONMENT {
            command.env(name, value);
        }
        // `exec` only returns on failure; carry on untuned rather than refuse to run.
        let _ = command.exec();
    }
}

/// A wrong condition here is a fork bomb: the marker must stop the second pass.
#[cfg_attr(not(all(unix, target_env = "gnu")), allow(dead_code))]
const fn wanted(marker: bool, disable: bool) -> bool {
    !marker && !disable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_stops_a_second_pass() {
        assert!(!wanted(true, false));
    }

    #[test]
    fn the_opt_out_stops_it() {
        assert!(!wanted(false, true));
    }

    #[test]
    fn a_first_run_with_no_opt_out_retunes() {
        assert!(wanted(false, false));
    }

    #[test]
    fn the_documented_variables_are_the_ones_applied() {
        let names: Vec<&str> = ENVIRONMENT.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            names,
            vec![
                "MALLOC_ARENA_MAX",
                "MALLOC_TRIM_THRESHOLD_",
                "MALLOC_MMAP_THRESHOLD_"
            ]
        );
        assert!(
            ENVIRONMENT
                .iter()
                .all(|(_, value)| value.parse::<u64>().is_ok()),
            "every value has to be a number glibc will parse"
        );
    }
}
