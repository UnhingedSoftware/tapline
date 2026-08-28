mod decode;
mod encode;

pub use decode::{Decoder, WireError};
pub use encode::Encoder;

pub const MAX_DEPTH: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    Varint,
    Fixed64,
    LengthDelimited,
    Fixed32,
}

impl WireType {
    pub const fn from_bits(bits: u32) -> Result<Self, WireError> {
        match bits {
            0 => Ok(Self::Varint),
            1 => Ok(Self::Fixed64),
            2 => Ok(Self::LengthDelimited),
            5 => Ok(Self::Fixed32),
            other => Err(WireError::InvalidWireType(other)),
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldKey {
    pub number: u32,
    pub wire_type: WireType,
}

pub trait Message: Sized + Default {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError>;

    fn encode_raw(&self, encoder: &mut Encoder);

    fn decode(bytes: &[u8]) -> Result<Self, WireError> {
        let mut message = Self::default();
        let mut decoder = Decoder::new(bytes);
        message.merge(&mut decoder)?;
        Ok(message)
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        let mut encoder = Encoder::new();
        self.encode_raw(&mut encoder);
        encoder.into_vec()
    }
}

pub trait Rpc: Message {
    type Response: Message;

    const TARGET: &'static str;
}

#[inline]
#[must_use]
pub const fn zigzag_encode_32(value: i32) -> u32 {
    ((value << 1) ^ (value >> 31)) as u32
}

#[inline]
#[must_use]
pub const fn zigzag_decode_32(value: u32) -> i32 {
    ((value >> 1) as i32) ^ -((value & 1) as i32)
}

#[inline]
#[must_use]
pub const fn zigzag_encode_64(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

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
        assert_eq!(zigzag_encode_32(0), 0);
        assert_eq!(zigzag_encode_32(-1), 1);
        assert_eq!(zigzag_encode_32(1), 2);
        assert_eq!(zigzag_encode_32(-2), 3);
        assert_eq!(zigzag_encode_32(2_147_483_647), 4_294_967_294);
        assert_eq!(zigzag_encode_32(-2_147_483_648), 4_294_967_295);
    }

    #[test]
    fn group_wire_types_are_rejected_rather_than_skipped() {
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
