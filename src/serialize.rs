use std::{borrow::Borrow, io::{Read, Write}};
use crate::{error::{BinverseError, BinverseResult}, streams::{Deserializer, Serializer}};


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

impl<T> SizedSerialize for T
where T: Borrow<str> {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        // TODO: slicing here could make code more inefficient when the size is usually correct
        s.write(self.borrow()[..size].as_bytes())
    }
    fn size(&self) -> usize { self.borrow().len() }
}
impl SizedDeserialize for String {
    fn deserialize<R: Read>(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self> {
        let mut b = vec![0; size];
        d.read(&mut b)?;
        String::from_utf8(b).or(Err(BinverseError::InvalidUTF8))
    }
}