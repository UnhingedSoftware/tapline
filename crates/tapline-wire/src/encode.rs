//! The encoder.
//!
//! Encoding is the easy direction — the values come from our own structs — so
//! this is deliberately small. The one subtlety is nested messages: protobuf
//! needs the length before the payload, and the length is not known until the
//! payload is written.

use crate::{FieldKey, Message, WireType};

/// A growable protobuf message buffer.
#[derive(Debug, Default, Clone)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    /// A new empty encoder.
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    /// A new encoder with room for `capacity` bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    /// The encoded bytes.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    /// The encoded bytes, borrowed.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    /// How many bytes have been written.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether nothing has been written.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Writes a base-128 varint.
    pub fn write_varint(&mut self, mut value: u64) {
        while value >= 0x80 {
            self.buf.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.buf.push(value as u8);
    }

    /// Writes a field header.
    pub fn write_key(&mut self, key: FieldKey) {
        self.write_varint((u64::from(key.number) << 3) | u64::from(key.wire_type.to_bits()));
    }

    /// Writes a varint field.
    pub fn write_varint_field(&mut self, number: u32, value: u64) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::Varint,
        });
        self.write_varint(value);
    }

    /// Writes an `int32` field.
    ///
    /// Negative values are sign-extended to ten bytes, which is protobuf's own
    /// encoding: it is wasteful, and it is what every other implementation
    /// expects, so `sint32` exists for anyone who minds.
    pub fn write_int32_field(&mut self, number: u32, value: i32) {
        self.write_varint_field(number, value as i64 as u64);
    }

    /// Writes a `bool` field.
    pub fn write_bool_field(&mut self, number: u32, value: bool) {
        self.write_varint_field(number, u64::from(value));
    }

    /// Writes a `fixed32` field.
    pub fn write_fixed32_field(&mut self, number: u32, value: u32) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::Fixed32,
        });
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    /// Writes a `fixed64` field.
    pub fn write_fixed64_field(&mut self, number: u32, value: u64) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::Fixed64,
        });
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    /// Writes a length-delimited field from bytes already in hand.
    pub fn write_bytes_field(&mut self, number: u32, value: &[u8]) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });
        self.write_varint(value.len() as u64);
        self.buf.extend_from_slice(value);
    }

    /// Writes a `string` field.
    pub fn write_string_field(&mut self, number: u32, value: &str) {
        self.write_bytes_field(number, value.as_bytes());
    }

    /// Writes a nested message field.
    ///
    /// The payload is written first with a one-byte length placeholder, then the
    /// real length is spliced in. That costs a memmove for messages longer than
    /// 127 bytes, and it avoids encoding the submessage twice — which is what
    /// the alternative (measure, then write) amounts to for anything with its
    /// own nested fields.
    pub fn write_message_field(&mut self, number: u32, message: &impl Message) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });

        let placeholder = self.buf.len();
        self.buf.push(0);
        let start = self.buf.len();

        message.encode_raw(self);

        let len = self.buf.len() - start;
        if len < 0x80 {
            // The common case: the placeholder byte was the right width.
            if let Some(slot) = self.buf.get_mut(placeholder) {
                *slot = len as u8;
            }
        } else {
            let mut encoded_len = Encoder::new();
            encoded_len.write_varint(len as u64);
            let extra = encoded_len.len() - 1;
            self.buf.splice(placeholder..=placeholder, encoded_len.buf);
            debug_assert_eq!(self.buf.len(), start + len + extra);
        }
    }

    /// Writes a packed repeated `fixed32` field, omitting it when empty.
    ///
    /// proto3 packs by default and an empty packed field is indistinguishable
    /// from an absent one, so writing a zero-length payload would be noise.
    pub fn write_packed_fixed32(&mut self, number: u32, values: &[u32]) {
        if values.is_empty() {
            return;
        }
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });
        self.write_varint((values.len() * 4) as u64);
        for value in values {
            self.buf.extend_from_slice(&value.to_le_bytes());
        }
    }

    /// Writes a packed repeated varint field, omitting it when empty.
    pub fn write_packed_varint(&mut self, number: u32, values: &[u64]) {
        if values.is_empty() {
            return;
        }
        let mut payload = Encoder::new();
        for value in values {
            payload.write_varint(*value);
        }
        self.write_bytes_field(number, payload.as_slice());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Decoder;

    #[test]
    fn varints_encode_the_specification_examples() {
        let mut e = Encoder::new();
        e.write_varint(0);
        e.write_varint(1);
        e.write_varint(300);
        assert_eq!(e.as_slice(), &[0x00, 0x01, 0xAC, 0x02]);
    }

    #[test]
    fn every_varint_round_trips() {
        for value in [0_u64, 1, 127, 128, 300, u32::MAX as u64, u64::MAX] {
            let mut e = Encoder::new();
            e.write_varint(value);
            assert_eq!(
                Decoder::new(e.as_slice()).read_varint(),
                Ok(value),
                "{value}"
            );
        }
    }

    #[test]
    fn negative_int32_uses_the_ten_byte_sign_extended_form() {
        // This is protobuf's own encoding for a negative int32; a shorter one
        // would not survive a round trip through any other implementation.
        let mut e = Encoder::new();
        e.write_int32_field(1, -1);
        assert_eq!(
            e.as_slice(),
            &[
                0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01
            ]
        );

        let mut d = Decoder::new(e.as_slice());
        let _ = d.read_key();
        assert_eq!(d.read_varint32().map(|v| v as i32), Ok(-1));
    }

    /// A message with a nested message, used to exercise the length placeholder
    /// on both sides of the 127-byte boundary.
    #[derive(Debug, Default, PartialEq, Eq)]
    struct Inner {
        payload: Vec<u8>,
    }

    impl Message for Inner {
        fn merge(&mut self, d: &mut Decoder<'_>) -> Result<(), crate::WireError> {
            while let Some(key) = d.read_key()? {
                if key.number == 1 {
                    self.payload = d.read_bytes()?.to_vec();
                } else {
                    d.skip_field(key.wire_type)?;
                }
            }
            Ok(())
        }

        fn encode_raw(&self, e: &mut Encoder) {
            e.write_bytes_field(1, &self.payload);
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct Outer {
        inner: Inner,
        tail: u64,
    }

    impl Message for Outer {
        fn merge(&mut self, d: &mut Decoder<'_>) -> Result<(), crate::WireError> {
            while let Some(key) = d.read_key()? {
                match key.number {
                    1 => {
                        let mut inner = Inner::default();
                        d.read_nested(|nested| inner.merge(nested))?;
                        self.inner = inner;
                    }
                    2 => self.tail = d.read_varint()?,
                    _ => d.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, e: &mut Encoder) {
            e.write_message_field(1, &self.inner);
            e.write_varint_field(2, self.tail);
        }
    }

    #[test]
    fn short_nested_message_round_trips() {
        let msg = Outer {
            inner: Inner {
                payload: b"hi".to_vec(),
            },
            tail: 7,
        };
        let encoded = msg.encode_to_vec();
        assert_eq!(Outer::decode(&encoded), Ok(msg));
    }

    #[test]
    fn nested_message_longer_than_the_placeholder_splices_correctly() {
        // 200 bytes forces a two-byte length varint, so the placeholder has to
        // grow and everything after it has to shift. Getting the splice wrong
        // corrupts the field that follows, which is why `tail` is checked too.
        let msg = Outer {
            inner: Inner {
                payload: vec![0xAB; 200],
            },
            tail: 12_345,
        };
        let encoded = msg.encode_to_vec();
        let decoded = Outer::decode(&encoded).expect("must round trip");
        assert_eq!(decoded.inner.payload.len(), 200);
        assert_eq!(decoded.tail, 12_345);
        assert_eq!(decoded, msg);
    }

    #[test]
    fn deeply_long_nested_messages_round_trip_at_every_length_boundary() {
        // 127/128 and 16383/16384 are where the length varint changes width.
        for len in [
            0_usize, 1, 126, 127, 128, 129, 16_382, 16_383, 16_384, 16_385,
        ] {
            let msg = Outer {
                inner: Inner {
                    payload: vec![0x5A; len],
                },
                tail: 99,
            };
            let decoded = Outer::decode(&msg.encode_to_vec()).expect("must round trip");
            assert_eq!(decoded.inner.payload.len(), len, "payload length {len}");
            assert_eq!(decoded.tail, 99, "trailing field corrupted at length {len}");
        }
    }

    #[test]
    fn empty_packed_fields_are_omitted_entirely() {
        let mut e = Encoder::new();
        e.write_packed_fixed32(1, &[]);
        e.write_packed_varint(2, &[]);
        assert!(e.is_empty());
    }

    #[test]
    fn packed_fields_round_trip() {
        let values = [1_u64, 300, u64::MAX];
        let mut e = Encoder::new();
        e.write_packed_varint(1, &values);

        let mut d = Decoder::new(e.as_slice());
        let _ = d.read_key();
        let mut out = Vec::new();
        d.read_packed_varint(&mut out).expect("must decode");
        assert_eq!(out, values);
    }
}
