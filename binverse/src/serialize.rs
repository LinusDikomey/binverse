use std::io::{Read, Write};
use crate::{error::BinverseResult, streams::{Deserializer, Serializer}};


pub trait Serialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()>;
}
pub trait Deserialize : Sized {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self>;
}

#[derive(Clone, Copy, Debug)]
pub enum SizeBytes {
    One,
    Two,
    Four,
    Eight,
    Var
}

pub trait SizedSerialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()>;
    fn size(&self) -> usize;
}
pub trait SizedDeserialize : Sized {
    fn deserialize<R: Read>(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self>;
}