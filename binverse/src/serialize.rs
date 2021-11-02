use std::io::{Read, Write};
use crate::{error::BinverseResult, streams::{Deserializer, Serializer}};


pub trait Serialize<W: Write> {
    fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()>;
}
pub trait Deserialize<R: Read> : Sized {
    fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self>;
}

#[derive(Clone, Copy, Debug)]
pub enum SizeBytes {
    One,
    Two,
    Four,
    Eight,
    Var
}
impl SizeBytes {
    #[cfg_attr(feature = "inline", inline)]
    pub fn to_str(&self) -> &'static str {
        use SizeBytes::*;
        match self {
            One => "One",
            Two => "Two",
            Four => "Four",
            Eight => "Eight",
            Var => "Var"
        }
    }
}

pub trait SizedSerialize<W> : Serialize<W>
where W: Write {
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()>;
    fn size(&self) -> usize;
}
pub trait SizedDeserialize<R> : Deserialize<R> + Sized
where R: Read {
    fn deserialize_sized(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self>;
}