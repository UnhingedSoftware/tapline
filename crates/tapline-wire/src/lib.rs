//! The protobuf wire format, implemented directly.
//!
//! This exists so nothing downstream inherits `prost`, a build script, or a
//! `protoc` binary. Code generation happens once in `xtask` and its output is
//! committed; at runtime the generated structs call straight into [`Decoder`]
//! and [`Encoder`] here.
//!
//! # Reading untrusted bytes
//!
//! Every byte this crate parses came off a socket. The decoder therefore:
//!
//! * never panics — `indexing_slicing` is denied workspace-wide, so a truncated
//!   message returns [`WireError::Truncated`] instead of slicing past the end;
//! * bounds every allocation by the remaining input, so a length prefix claiming
//!   4 GB inside a 100-byte message cannot make us reserve 4 GB;
//! * bounds nesting depth, so a message that is nothing but nested
//!   length-delimited headers cannot blow the stack.
//!
//! # Scope
//!
//! Steam's `.proto` files are proto2. The subset used here is what Steam
//! actually puts on the wire: varints, 32- and 64-bit fixed fields,
//! length-delimited fields, and packed repeated scalars. Groups (wire types 3
//! and 4) were removed from protobuf before Steam's schema was written and are
//! rejected rather than skipped, because silently ignoring a field we do not
//! understand is how a decoder ends up disagreeing with the sender about what a
//! message said.

mod decode;
mod encode;

pub use decode::{Decoder, WireError};
pub use encode::Encoder;

/// How many nested length-delimited messages the decoder will follow.
///
/// Steam's own messages nest three or four deep. The limit exists for hostile
/// input, not legitimate input, so it is set far above what the protocol needs
/// and far below what would exhaust the stack.
pub const MAX_DEPTH: u32 = 64;

/// A protobuf wire type: the low three bits of a field key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    /// Base-128 varint: `int32`, `int64`, `uint32`, `uint64`, `sint*`, `bool`, enums.
    Varint,
    /// Fixed 64 bits: `fixed64`, `sfixed64`, `double`.
    Fixed64,
    /// Length-prefixed bytes: `string`, `bytes`, embedded messages, packed repeated.
    LengthDelimited,
    /// Fixed 32 bits: `fixed32`, `sfixed32`, `float`.
    Fixed32,
}

impl WireType {
    /// Decodes the low three bits of a field key.
    ///
    /// Wire types 3 and 4 are the deprecated group markers, and 6 and 7 have
    /// never been assigned; all are rejected.
    pub const fn from_bits(bits: u32) -> Result<Self, WireError> {
        match bits {
            0 => Ok(Self::Varint),
            1 => Ok(Self::Fixed64),
            2 => Ok(Self::LengthDelimited),
            5 => Ok(Self::Fixed32),
            other => Err(WireError::InvalidWireType(other)),
        }
    }

    /// The low three bits of a field key.
    #[must_use]
    pub const fn to_bits(self) -> u32 {
        match self {
            Self::Varint => 0,
            Self::Fixed64 => 1,
            Self::LengthDelimited => 2,
            Self::Fixed32 => 5,
        }
    }
}

/// One field header: its number and how its value is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldKey {
    /// The field number from the `.proto` definition.
    pub number: u32,
    /// How the value that follows is encoded.
    pub wire_type: WireType,
}

/// A protobuf message that can be read from and written to the wire.
///
/// Generated code implements this; hand-written code should not need to.
pub trait Message: Sized + Default {
    /// Merges the fields in `decoder` into `self`.
    ///
    /// Merging rather than replacing is protobuf's own semantics: a repeated
    /// field appends, and a message field merges recursively. It also means a
    /// message split across two length-delimited chunks reassembles correctly.
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError>;

    /// Appends this message's fields to `encoder`.
    fn encode_raw(&self, encoder: &mut Encoder);

    /// Decodes a complete message from `bytes`.
    fn decode(bytes: &[u8]) -> Result<Self, WireError> {
        let mut message = Self::default();
        let mut decoder = Decoder::new(bytes);
        message.merge(&mut decoder)?;
        Ok(message)
    }

    /// Encodes this message into a new buffer.
    fn encode_to_vec(&self) -> Vec<u8> {
        let mut encoder = Encoder::new();
        self.encode_raw(&mut encoder);
        encoder.into_vec()
    }
}

/// A request message that names its own response type and RPC target.
///
/// Steam's unified messages are addressed by a string such as
/// `Authentication.BeginAuthSessionViaCredentials`, and every one of them has
/// exactly one response type. Tying the three together in the type system means
/// the RPC layer can be written once, generically, and a caller cannot ask for
/// one method and decode another's reply — a mistake that otherwise surfaces as
/// a message that decodes to all-default fields with no error at all.
///
/// Implemented by generated code from the `service` blocks in Valve's schema.
pub trait Rpc: Message {
    /// The reply Steam sends.
    type Response: Message;

    /// The target string, without the `#1` version suffix the transport adds.
    const TARGET: &'static str;
}

/// Zigzag-encodes a signed 32-bit value for a `sint32` field.
#[inline]
#[must_use]
pub const fn zigzag_encode_32(value: i32) -> u32 {
    ((value << 1) ^ (value >> 31)) as u32
}

/// Reverses [`zigzag_encode_32`].
#[inline]
#[must_use]
pub const fn zigzag_decode_32(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}

/// Zigzag-encodes a signed 64-bit value for a `sint64` field.
#[inline]
#[must_use]
pub const fn zigzag_encode_64(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

/// Reverses [`zigzag_encode_64`].
#[inline]
#[must_use]
pub const fn zigzag_decode_64(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zigzag_round_trips_across_the_sign_boundary() {
        for value in [0_i32, -1, 1, i32::MIN, i32::MAX, -2_147_483_647] {
            assert_eq!(
                zigzag_decode_32(zigzag_encode_32(value)),
                value,
                "i32 {value}"
            );
        }
        for value in [0_i64, -1, 1, i64::MIN, i64::MAX] {
            assert_eq!(
                zigzag_decode_64(zigzag_encode_64(value)),
                value,
                "i64 {value}"
            );
        }
    }

    #[test]
    fn zigzag_matches_the_specification_examples() {
        // From the protobuf encoding documentation.
        assert_eq!(zigzag_encode_32(0), 0);
        assert_eq!(zigzag_encode_32(-1), 1);
        assert_eq!(zigzag_encode_32(1), 2);
        assert_eq!(zigzag_encode_32(-2), 3);
        assert_eq!(zigzag_encode_32(2_147_483_647), 4_294_967_294);
        assert_eq!(zigzag_encode_32(-2_147_483_648), 4_294_967_295);
    }

    #[test]
    fn group_wire_types_are_rejected_rather_than_skipped() {
        // A decoder that skipped these would silently disagree with the sender
        // about the contents of the message.
        assert!(matches!(
            WireType::from_bits(3),
            Err(WireError::InvalidWireType(3))
        ));
        assert!(matches!(
            WireType::from_bits(4),
            Err(WireError::InvalidWireType(4))
        ));
        assert!(matches!(
            WireType::from_bits(6),
            Err(WireError::InvalidWireType(6))
        ));
        assert!(matches!(
            WireType::from_bits(7),
            Err(WireError::InvalidWireType(7))
        ));
    }

    #[test]
    fn wire_types_round_trip() {
        for wt in [
            WireType::Varint,
            WireType::Fixed64,
            WireType::LengthDelimited,
            WireType::Fixed32,
        ] {
            assert_eq!(WireType::from_bits(wt.to_bits()), Ok(wt));
        }
    }
}
