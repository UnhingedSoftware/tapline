//! Keeping the allocator from hoarding.
//!
//! A download decrypts and decompresses up to a megabyte per chunk and frees it
//! again, thousands of times. glibc watches that pattern, concludes blocks that
//! size are worth keeping, raises its dynamic mmap threshold, and stops
//! returning them to the kernel. Peak memory then reflects what the allocator
//! decided to retain rather than what the download actually had in flight.
//!
//! The size of the effect, measured on the same 1.5 GB install:
//!
//! | | peak RSS |
//! |---|---|
//! | pinned | 21–24 MB |
//! | left alone | 46–57 MB |
//!
//! Left alone it also drifts by around 20% between identical runs, because the
//! threshold lands somewhere different depending on how the frees interleaved.
//! Pinned, repeats agree to within a megabyte.
//!
//! # Why this is a re-exec and not a function call
//!
//! The knobs are `mallopt` parameters. Setting them properly means calling
//! `mallopt` before the first allocation, which is unreachable from safe Rust
//! and would mean unsafe FFI in a crate that forbids it. glibc also reads them
//! from the environment at startup, which is reachable — but only at startup,
//! which has already happened by the time any Rust code runs.
//!
//! So the process replaces itself, once, with the variables set. It costs one
//! `execve` — a millisecond, measured — and nothing else.
//!
//! # This is for programs, not for libraries
//!
//! [`retune`] replaces the running process. That is fine for a binary whose
//! `main` calls it as its first statement, and catastrophic anywhere else: a
//! library that did it would restart its host, and a plugin that did it would
//! restart whatever loaded it. It is not called automatically for that reason.
//!
//! Anything embedding tapline — the FFI, a server holding a session open —
//! should set the variables on the process instead, before it starts:
//!
//! ```sh
//! MALLOC_ARENA_MAX=2 MALLOC_TRIM_THRESHOLD_=131072 MALLOC_MMAP_THRESHOLD_=131072
//! ```
//!
//! [`ENVIRONMENT`] is that list, so a launcher can apply it without copying the
//! numbers out of this documentation and letting them drift.

/// The allocator settings a download wants, as name/value pairs.
///
/// Exposed so an embedder can apply them to a process it is about to spawn, and
/// so the test below can check [`retune`] and the documentation agree.
///
/// - `MALLOC_ARENA_MAX` caps per-thread arenas. A download's threads all
///   allocate the same shapes, so extra arenas fragment rather than help.
/// - `MALLOC_TRIM_THRESHOLD_` is how much free top-of-heap to keep before
///   returning it.
/// - `MALLOC_MMAP_THRESHOLD_` is the one that matters: pinning it stops glibc
///   raising it dynamically, so chunk-sized blocks keep going back to the
///   kernel instead of being retained.
pub const ENVIRONMENT: [(&str, &str); 3] = [
    ("MALLOC_ARENA_MAX", "2"),
    ("MALLOC_TRIM_THRESHOLD_", "131072"),
    ("MALLOC_MMAP_THRESHOLD_", "131072"),
];

/// The marker that says the re-exec already happened.
///
/// Inherited by the replacement process, which is what stops [`retune`]
/// recursing.
pub const MARKER: &str = "TAPLINE_TUNED";

/// The opt-out.
pub const DISABLE: &str = "TAPLINE_NO_MALLOC_TUNING";

/// Re-runs the current program once with the allocator pinned.
///
/// Call it as the first statement of `main`, before starting a runtime or
/// allocating anything worth keeping — everything the process has done so far
/// is discarded by the exec.
///
/// Returns normally, having done nothing, when the re-exec already happened,
/// when `TAPLINE_NO_MALLOC_TUNING` is set, or when the exec fails. A program
/// that cannot re-exec should still run, just with the memory profile it would
/// have had anyway.
///
/// # Panics
///
/// Never. It has no failure path a caller could act on.
///
/// # Platform
///
/// Does nothing outside Unix, and nothing useful outside glibc — musl has no
/// dynamic mmap threshold to pin, so there is nothing to fix there.
pub fn retune() {
    // glibc only. musl's allocator has no dynamic mmap threshold to pin and
    // ignores these variables, so on a musl build — which is what the container
    // image is — this would be an `execve` that bought nothing.
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
        // `exec` only returns on failure, and the failure is not actionable:
        // carry on untuned rather than refuse to run.
        let _ = command.exec();
    }
}

/// Whether a re-exec should happen, given what the environment says.
///
/// Split out from [`retune`] because the alternative way to test the condition
/// is to set environment variables, which is `unsafe` in this edition and
/// forbidden in this workspace — and because getting it wrong is not an
/// ordinary bug. `retune` replaces the process, so a condition that failed to
/// stop the second pass would be a fork bomb.
// Only [`retune`] calls it, and that body is glibc-only — but the tests below
// exercise it on any target, which is the point of splitting it out.
#[cfg_attr(not(all(unix, target_env = "gnu")), allow(dead_code))]
const fn wanted(marker: bool, disable: bool) -> bool {
    !marker && !disable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_stops_a_second_pass() {
        // The property that keeps this from being a fork bomb.
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
        // The module documentation tells an embedder to set these by hand. If
        // the list here changed and that text did not, the advice would be
        // wrong in a way nothing else would catch.
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
