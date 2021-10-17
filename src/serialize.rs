use std::{borrow::Borrow, io::{Read, Write}};
use crate::streams::{Deserializer, Serializer};


pub trait Serialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>);
}
pub trait Deserialize {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> Self;
}

#[derive(Clone, Copy)]
pub enum SizeBytes {
    One,
    Two,
    Four,
    Eight
}

pub trait SizedSerialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize);
    fn size(&self) -> usize;
}
pub trait SizedDeserialize {
    fn deserialize<R: Read>(d: &mut Deserializer<R>, size: usize) -> Self;
}

impl<T> SizedSerialize for T
where T: Borrow<str> {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize) {
        // TODO: slicing here could make code more inefficient when the size is usually correct
        s.write(self.borrow()[..size].as_bytes())
    }
    fn size(&self) -> usize { self.borrow().len() }
}
impl SizedDeserialize for String {
    fn deserialize<R: Read>(d: &mut Deserializer<R>, size: usize) -> Self {
        let mut b = vec![0; size];
        d.read(&mut b);
        String::from_utf8(b).unwrap()
    }
}