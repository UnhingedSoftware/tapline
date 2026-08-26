//! Anything the encoder writes, the decoder must read back unchanged.
//!
//! The decode-only target proves we survive hostile input. This one proves the
//! two halves agree with each other, which is the failure that would otherwise
//! surface as Steam rejecting a logon for reasons no log line explains.

#![no_main]

use libfuzzer_sys::fuzz_target;
use tapline_wire::{Decoder, Encoder, WireType};

fuzz_target!(|values: Vec<u64>| {
    // Varints: the interesting boundaries are the seven-bit steps, and a fuzzer
    // finds them faster than a hand-written table does.
    let mut encoder = Encoder::new();
    for value in &values {
        encoder.write_varint(*value);
    }
    let encoded = encoder.into_vec();

    let mut decoder = Decoder::new(&encoded);
    for expected in &values {
        match decoder.read_varint() {
            Ok(got) => assert_eq!(got, *expected, "varint round trip disagreed"),
            Err(e) => panic!("failed to read back a varint we wrote: {e}"),
        }
    }
    assert!(decoder.is_empty(), "decoder left bytes unread after our own encoding");

    // Length-delimited fields, where the encoder's length placeholder has to
    // grow for payloads past 127 bytes.
    let payload: Vec<u8> = values.iter().map(|v| *v as u8).collect();
    let mut encoder = Encoder::new();
    encoder.write_bytes_field(1, &payload);
    encoder.write_varint_field(2, 0xDEAD_BEEF);
    let encoded = encoder.into_vec();

    let mut decoder = Decoder::new(&encoded);
    let key = decoder.read_key().expect("our own key must parse").expect("a field must be present");
    assert_eq!(key.number, 1);
    assert_eq!(key.wire_type, WireType::LengthDelimited);
    assert_eq!(decoder.read_bytes().expect("our own payload must parse"), &payload[..]);

    // The field after a spliced length is where an off-by-one shows up.
    let key = decoder.read_key().expect("trailing key must parse").expect("a field must follow");
    assert_eq!(key.number, 2, "the field after the payload was corrupted");
    assert_eq!(decoder.read_varint(), Ok(0xDEAD_BEEF));
});
