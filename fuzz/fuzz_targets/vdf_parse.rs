//! KeyValues parsing must survive arbitrary text, and anything that parses must
//! survive a write/reparse round trip unchanged.
//!
//! The round-trip half is the one that matters operationally: tapline shares
//! `appmanifest` files with the Steam client, so a document that changes shape
//! when rewritten is a file we would corrupt on every update.

#![no_main]

use libfuzzer_sys::fuzz_target;
use tapline_vdf::parse;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    let Ok(parsed) = parse(text) else {
        // Rejecting malformed input is the expected outcome, not a failure.
        return;
    };

    // Whatever we accepted, we must be able to write and read back identically.
    let written = parsed.to_string();
    match parse(&written) {
        Ok(reparsed) => assert_eq!(
            parsed, reparsed,
            "a document changed meaning when written and read back"
        ),
        Err(e) => panic!("we wrote a document we cannot read: {e}"),
    }

    // And writing it a second time must produce the same bytes, or an update
    // would rewrite files it did not mean to touch.
    assert_eq!(written, parse(&written).map(|o| o.to_string()).unwrap_or_default());
});
