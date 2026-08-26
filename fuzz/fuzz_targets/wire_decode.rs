//! The decoder must never panic, hang, or exhaust memory on arbitrary bytes.
//!
//! This is the shape of input the decoder actually gets: whatever arrived on
//! the socket. The target asserts nothing about the *result* — a hostile message
//! is supposed to produce an error — only that producing it is survivable.

#![no_main]

use libfuzzer_sys::fuzz_target;
use tapline_wire::{Decoder, WireError, WireType};

/// Walks a message the way generated code does, descending into every nested
/// field, so the depth limit and the length checks are both on the path.
fn walk(decoder: &mut Decoder<'_>) -> Result<(), WireError> {
    while let Some(key) = decoder.read_key()? {
        match key.wire_type {
            WireType::LengthDelimited => {
                // A length-delimited field is ambiguous on the wire: it could be
                // a nested message, a string, or packed scalars. Trying it as a
                // message first is what a real decoder does for a field it knows
                // is a message, so that is the path worth fuzzing.
                decoder.read_nested(walk)?;
            }
            other => decoder.skip_field(other)?,
        }
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let mut decoder = Decoder::new(data);
    let _ = walk(&mut decoder);

    // The scalar readers are reachable directly from generated code, so they get
    // their own pass over the same bytes rather than only being exercised
    // through skip_field.
    let mut scalars = Decoder::new(data);
    while let Ok(Some(key)) = scalars.read_key() {
        let stop = match key.wire_type {
            WireType::Varint => scalars.read_varint().is_err(),
            WireType::Fixed32 => scalars.read_fixed32().is_err(),
            WireType::Fixed64 => scalars.read_fixed64().is_err(),
            WireType::LengthDelimited => {
                // read_string shares read_bytes' bounds check and adds UTF-8
                // validation on top.
                scalars.read_string().is_err() && scalars.read_bytes().is_err()
            }
        };
        if stop {
            break;
        }
    }
});
