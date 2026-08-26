//! The header and the Rust signatures must agree.
//!
//! A C header is the part of the interface a consumer actually reads, and it is
//! the part nothing checks. Change a Rust `extern "C"` signature, forget the
//! header, and the next C or koffi caller passes the old argument list — which
//! is not a compile error anywhere, it is a corrupted stack at run time.
//!
//! Two halves, and both are needed:
//!
//! * assigning each function to an explicitly typed function pointer, so a
//!   changed Rust signature fails to compile here;
//! * parsing the header and comparing its parameter counts to the same list,
//!   so a Rust change that was not mirrored into the header fails the test.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::ffi::c_char;
use tapline_ffi::TaplineJob;

/// Every exported function, with the arity the header must declare.
///
/// The type ascriptions are the real check: if a Rust signature changes, this
/// file stops compiling, and whoever changed it is standing right next to the
/// header they also need to change.
fn expected() -> Vec<(&'static str, usize)> {
    let install: unsafe extern "C" fn(
        u32,
        *const c_char,
        *const c_char,
        u32,
        u8,
        u8,
        u8,
        u8,
        *mut *mut TaplineJob,
    ) -> i32 = tapline_ffi::tapline_install;

    let plan: unsafe extern "C" fn(
        u32,
        *const c_char,
        *const c_char,
        u8,
        u8,
        *mut *mut TaplineJob,
    ) -> i32 = tapline_ffi::tapline_plan;

    let workshop: unsafe extern "C" fn(u32, u64, *const c_char, u32, *mut *mut TaplineJob) -> i32 =
        tapline_ffi::tapline_workshop_download;

    let next: unsafe extern "C" fn(*mut TaplineJob, u32, *mut u8, usize, *mut usize) -> i32 =
        tapline_ffi::tapline_job_next;

    let cancel: unsafe extern "C" fn(*mut TaplineJob) = tapline_ffi::tapline_job_cancel;
    let free: unsafe extern "C" fn(*mut TaplineJob) = tapline_ffi::tapline_job_free;
    let last_error: unsafe extern "C" fn(*mut u8, usize, *mut usize) -> i32 =
        tapline_ffi::tapline_last_error;
    let version: extern "C" fn() -> *const c_char = tapline_ffi::tapline_version;

    // Referenced so the bindings above are not dead code; the ascriptions are
    // what this function exists for.
    let _ = (
        install, plan, workshop, next, cancel, free, last_error, version,
    );

    vec![
        ("tapline_install", 9),
        ("tapline_plan", 6),
        ("tapline_workshop_download", 5),
        ("tapline_job_next", 5),
        ("tapline_job_cancel", 1),
        ("tapline_job_free", 1),
        ("tapline_last_error", 3),
        ("tapline_version", 0),
    ]
}

/// The declared parameter count for `name`, read out of the header.
fn header_arity(header: &str, name: &str) -> Option<usize> {
    // Find the declaration rather than a mention in a comment: a declaration is
    // the name immediately followed by `(`.
    let needle = format!("{name}(");
    let start = header.find(&needle)?;
    let open = start + needle.len();
    let rest = header.get(open..)?;
    let close = rest.find(')')?;
    let params = rest.get(..close)?.trim();

    if params.is_empty() || params == "void" {
        return Some(0);
    }
    Some(params.split(',').count())
}

#[test]
fn the_header_declares_every_exported_function() {
    let header = include_str!("../include/tapline.h");
    for (name, arity) in expected() {
        let declared = header_arity(header, name)
            .unwrap_or_else(|| panic!("{name} is not declared in include/tapline.h"));
        assert_eq!(
            declared, arity,
            "{name} takes {arity} parameters in Rust but {declared} in the header"
        );
    }
}

#[test]
fn the_header_declares_nothing_that_does_not_exist() {
    // The other direction: a function removed from Rust but left in the header
    // is a consumer calling into nothing.
    let header = include_str!("../include/tapline.h");
    let known: Vec<&str> = expected().into_iter().map(|(name, _)| name).collect();

    for line in header.lines() {
        let line = line.trim();
        // Declarations only: they end in `;` and mention a tapline_ name.
        if !line.ends_with(';') || !line.contains("tapline_") {
            continue;
        }
        let Some(start) = line.find("tapline_") else {
            continue;
        };
        let name: String = line
            .get(start..)
            .unwrap_or_default()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        assert!(
            known.contains(&name.as_str()),
            "the header declares {name}, which no longer exists"
        );
    }
}

#[test]
fn the_return_codes_match_the_rust_constants() {
    // These are copied by hand into the header, and a consumer branching on the
    // wrong number reads "done" as "buffer too small".
    let header = include_str!("../include/tapline.h");
    for (name, value) in [
        ("TAPLINE_OK", tapline_ffi::TAPLINE_OK),
        ("TAPLINE_TIMEOUT", tapline_ffi::TAPLINE_TIMEOUT),
        ("TAPLINE_DONE", tapline_ffi::TAPLINE_DONE),
        (
            "TAPLINE_BUFFER_TOO_SMALL",
            tapline_ffi::TAPLINE_BUFFER_TOO_SMALL,
        ),
        ("TAPLINE_BAD_ARGUMENT", tapline_ffi::TAPLINE_BAD_ARGUMENT),
    ] {
        let needle = format!("#define {name} {value}");
        assert!(header.contains(&needle), "the header is missing `{needle}`");
    }
}
