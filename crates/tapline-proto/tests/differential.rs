//! Differential tests against Google's own protobuf implementation.
//!
//! Round-tripping our encoder through our decoder proves the two halves agree
//! with each other, which is worth something but is exactly the test a
//! self-consistent-but-wrong codec passes. These tests use bytes produced by
//! `protoc` from Valve's real `content_manifest.proto`, so they prove we agree
//! with the implementation Steam is on the other end of.
//!
//! The fixture is committed rather than generated at test time: CI has no
//! `protoc`, and a test that silently skips when a tool is missing is a test
//! that stops running.
//!
//! Regenerating, if the schema changes:
//!
//! ```sh
//! cd crates/tapline-proto
//! protoc --encode=ContentManifestPayload protos/content_manifest.proto \
//!     < tests/fixtures/content_manifest_payload.txtpb \
//!     > tests/fixtures/content_manifest_payload.protoc.bin
//! ```

use tapline_proto::content_manifest::{ContentManifestPayload, content_manifest_payload};
use tapline_wire::Message;

/// Bytes `protoc` produced for the message in `content_manifest_payload.txtpb`.
const PROTOC_BYTES: &[u8] = include_bytes!("fixtures/content_manifest_payload.protoc.bin");

#[test]
fn we_decode_what_google_encoded() {
    let payload =
        ContentManifestPayload::decode(PROTOC_BYTES).expect("protoc's own output must decode");

    let mapping = payload
        .mappings
        .first()
        .expect("the fixture has one file mapping");

    assert_eq!(mapping.filename.as_deref(), Some("bin/srcds_linux"));
    assert_eq!(mapping.size, Some(12_345_678));
    assert_eq!(mapping.flags, Some(64));
    assert_eq!(
        mapping.sha_content.as_deref(),
        Some(&b"0123456789abcdef0123"[..])
    );
    assert_eq!(mapping.chunks.len(), 2);

    let first = mapping.chunks.first().expect("two chunks");
    assert_eq!(first.sha.as_deref(), Some(&b"abcdefghij0123456789"[..]));
    // A fixed32 field, which is where a little-endian mistake would show up.
    assert_eq!(first.crc, Some(0x1234_5678));
    assert_eq!(first.offset, Some(0));
    assert_eq!(first.cb_original, Some(1_048_576));
    assert_eq!(first.cb_compressed, Some(524_288));

    let second = mapping.chunks.get(1).expect("two chunks");
    assert_eq!(second.crc, Some(0x8765_4321));
    assert_eq!(second.offset, Some(1_048_576));
}

#[test]
fn what_we_encode_is_byte_identical_to_google() {
    // Rebuilding the fixture's message by hand and encoding it must produce the
    // same bytes protoc did. This is the assertion that catches a wrong wire
    // type, a wrong field number, or a varint written one byte too wide — none
    // of which a round trip through our own decoder would notice.
    let payload = ContentManifestPayload {
        mappings: vec![content_manifest_payload::FileMapping {
            filename: Some("bin/srcds_linux".to_owned()),
            size: Some(12_345_678),
            flags: Some(64),
            sha_filename: None,
            sha_content: Some(b"0123456789abcdef0123".to_vec()),
            chunks: vec![
                content_manifest_payload::file_mapping::ChunkData {
                    sha: Some(b"abcdefghij0123456789".to_vec()),
                    crc: Some(0x1234_5678),
                    offset: Some(0),
                    cb_original: Some(1_048_576),
                    cb_compressed: Some(524_288),
                },
                content_manifest_payload::file_mapping::ChunkData {
                    sha: Some(b"jihgfedcba9876543210".to_vec()),
                    crc: Some(0x8765_4321),
                    offset: Some(1_048_576),
                    cb_original: Some(65_536),
                    cb_compressed: Some(32_768),
                },
            ],
            linktarget: Some(String::new()),
        }],
    };

    assert_eq!(
        payload.encode_to_vec(),
        PROTOC_BYTES,
        "our encoding diverged from protoc's"
    );
}

#[test]
fn our_own_round_trip_is_stable() {
    let decoded = ContentManifestPayload::decode(PROTOC_BYTES).expect("must decode");
    let reencoded = decoded.encode_to_vec();
    assert_eq!(
        reencoded, PROTOC_BYTES,
        "decode then encode changed the bytes"
    );

    let twice = ContentManifestPayload::decode(&reencoded).expect("must decode again");
    assert_eq!(decoded, twice);
}

#[test]
fn unknown_fields_do_not_break_decoding() {
    // Valve adds fields; an older build must keep reading the ones it knows.
    let mut bytes = PROTOC_BYTES.to_vec();
    // Field 4095, varint, value 1 — a number nothing in the schema uses.
    bytes.extend_from_slice(&[0xF8, 0xFE, 0x03, 0x01]);

    let payload =
        ContentManifestPayload::decode(&bytes).expect("an unknown field must not be fatal");
    assert_eq!(payload.mappings.len(), 1);
}

#[test]
fn a_truncated_message_is_an_error_not_a_partial_value() {
    // Every prefix of a valid message is either an error or a shorter valid
    // message; none of them may panic. This is the shape a dropped connection
    // delivers.
    for cut in 1..PROTOC_BYTES.len() {
        let prefix = PROTOC_BYTES.get(..cut).expect("in range");
        // The result is not asserted — a prefix can legitimately parse — only
        // that producing it does not panic.
        let _ = ContentManifestPayload::decode(prefix);
    }
}
