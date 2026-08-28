#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::ffi::c_char;
use tapline_ffi::TaplineJob;

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
        *const c_char,
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

    let workshop: unsafe extern "C" fn(
        u32,
        u64,
        *const c_char,
        u32,
        u8,
        *const c_char,
        u8,
        *mut *mut TaplineJob,
    ) -> i32 = tapline_ffi::tapline_workshop_download;

    let next: unsafe extern "C" fn(*mut TaplineJob, u32, *mut u8, usize, *mut usize) -> i32 =
        tapline_ffi::tapline_job_next;

    let cancel: unsafe extern "C" fn(*mut TaplineJob) = tapline_ffi::tapline_job_cancel;
    let free: unsafe extern "C" fn(*mut TaplineJob) = tapline_ffi::tapline_job_free;
    let last_error: unsafe extern "C" fn(*mut u8, usize, *mut usize) -> i32 =
        tapline_ffi::tapline_last_error;
    let qr_login: unsafe extern "C" fn(u32, *mut *mut TaplineJob) -> i32 =
        tapline_ffi::tapline_qr_login;
    let version: extern "C" fn() -> *const c_char = tapline_ffi::tapline_version;
    let set_total: extern "C" fn(u32) -> i32 = tapline_ffi::tapline_set_total_concurrency;
    let total: extern "C" fn() -> u32 = tapline_ffi::tapline_total_concurrency;
    let available: extern "C" fn() -> u32 = tapline_ffi::tapline_available_concurrency;

    let _ = (
        install, plan, workshop, qr_login, next, cancel, free, last_error, version, set_total,
        total, available,
    );

    vec![
        ("tapline_install", 10),
        ("tapline_plan", 6),
        ("tapline_workshop_download", 8),
        ("tapline_workshop_search", 19),
        ("tapline_pipeline", 5),
        ("tapline_qr_login", 2),
        ("tapline_job_next", 5),
        ("tapline_job_cancel", 1),
        ("tapline_job_free", 1),
        ("tapline_last_error", 3),
        ("tapline_version", 0),
        ("tapline_set_total_concurrency", 1),
        ("tapline_total_concurrency", 0),
        ("tapline_available_concurrency", 0),
    ]
}

fn header_arity(header: &str, name: &str) -> Option<usize> {
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
    let header = include_str!("../include/tapline.h");
    let known: Vec<&str> = expected().into_iter().map(|(name, _)| name).collect();

    for line in header.lines() {
        let line = line.trim();
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
