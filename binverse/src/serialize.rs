use std::io::{Read, Write};
use crate::{error::BinverseResult, streams::{Deserializer, Serializer}};

/// The Serialize trait provides a function to serialize into a data stream.
/// It can be implemented manually or by using the #\[binverse_derive::serializable\] attribute.
pub trait Serialize<W: Write> {
    /// The serialize function.
    /// Arguments:
    /// - `s` - The serializer that the data will be written to.
    fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()>;
}

/// The deserialize trait provides a function to deserialize from a data
/// stream. It can be implemented manually or by using the #\[binverse_derive::serializable\]
/// attribute.
pub trait Deserialize<R: Read> : Sized {
    /// The deserialize function.
    /// Arguments:
    /// - `d` - The deserializer that the data will be read from.
    fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self>;
}

/// An enum representing the possible lengths of the size bytes for a variable
/// length data structure.
#[derive(Clone, Copy, Debug)]
pub enum SizeBytes {
    /// The length is serialized using a single byte ([u8]).
    One,
    /// The length is serialized using two bytes ([u16]).
    Two,
    /// The length is serialized using four bytes ([u32]).
    Four,
    /// The length is serialized using eight bytes ([u64]).
    Eight,
    /// The length is serialized using a variable amount of bytes depending on
    /// the size of the length. Larger numbers will take more bytes. This is
    /// often the default when no size bytes are specified. Note that
    /// serializing as a VarInt might decrease performance, so providing a size
    /// whenever the maximum size is known is recommended.
    Var
}
impl SizeBytes {
    /// Converts the enum variants to it's name as a [str]. This is used for
    /// debugging and macro implementations.
    pub const fn to_str(&self) -> &'static str {
        use SizeBytes::*;
        match self {
            One => "One",
            Two => "Two",
            Four => "Four",
            Eight => "Eight",
            Var => "Var"
        }
    }

    /// Returns the maximum possible length available with a SizeBytes variant.
    pub const fn maximum(&self) -> u64 {
        use SizeBytes::*;
        match self {
            One => u8::MAX as u64,
            Two => u16::MAX as u64,
            Four => u32::MAX as u64,
            Eight | Var => u64::MAX,
        }
    }
}

/// Similar to the [Serialize] trait, but for data structures with a variable
/// length, like arrays, [Vec]s, and [String]s.
pub trait SizedSerialize<W> : Serialize<W>
where W: Write {
    /// Writes the elements of the data structure up to the size.
    /// To write the length automatically, use the [crate::streams::Serializer::serialize_sized] function.
    /// Arguments:
    /// - `s` - The serializer that the data will be written to.
    /// - `size` - The number of elements to write.
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()>;

    /// Should return the current number of elements of the data structure.
    fn size(&self) -> usize;
}

/// Similar to the [Deserialize] trait, but for data structures with a variable
/// length, like arrays, [Vec]s, and [String]s.
pub trait SizedDeserialize<R> : Deserialize<R> + Sized
where R: Read {
    /// Reads `size` elements into a new instance of the data structure.
    /// To read a length stored in the data being deserialize, use [crate::streams::Deserializer::deserialize_sized].
    /// Arguments:
    /// - `d` - The deserializer that the data will be written to.
    /// - `size` - The number of elements to read
    fn deserialize_sized(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self>;
}