use std::{io::{Read, Write}};

use crate::serialize::{Deserialize, Serialize, SizeBytes, SizedDeserialize, SizedSerialize};

pub struct Serializer<W: Write> {
    w: W,
    revision: u32
}
impl<W: Write> Serializer<W> {
    pub fn new(w: W, revision: u32) -> Self {
        let mut s = Self { w, revision };
        revision.serialize(&mut s);
        s
    }
    pub fn write(&mut self, buf: &[u8]) {
        self.w.write_all(buf).unwrap();
    }
    pub fn write_size(&mut self, sb: SizeBytes, size: usize) {
        match sb {
            SizeBytes::One   => { debug_assert!(size as  u8 <=  u8::MAX); (size as  u8).serialize(self) },
            SizeBytes::Two   => { debug_assert!(size as u16 <= u16::MAX); (size as u16).serialize(self) },
            SizeBytes::Four  => { debug_assert!(size as u32 <= u32::MAX); (size as u32).serialize(self) },
            SizeBytes::Eight => { debug_assert!(size as u64 <= u64::MAX); (size as u64).serialize(self) },
        }
    }
    pub fn serialize_sized<T: SizedSerialize>(&mut self, sb: SizeBytes, t: &T) {
        let size = t.size();
        self.write_size(sb, size);
        t.serialize(self, size);
    }
    pub fn revision(&self) -> u32 { self.revision }
    pub fn finish(self) -> W { self.w }
}

pub struct Deserializer<R: Read> {
    r: R,
    revision: u32
}

impl<R: Read> Deserializer<R> {
    pub fn new(r: R) -> Self {
        let mut d = Self {
            r,
            revision: 0
        };
        d.revision = d.deserialize();
        d
    }
    pub fn read(&mut self, buf: &mut [u8]) {
        self.r.read_exact(buf).unwrap();
    }
    pub fn read_size(&mut self, sb: SizeBytes) -> usize  {
        match sb {
            SizeBytes::One   => self.deserialize::< u8>() as usize,
            SizeBytes::Two   => self.deserialize::<u16>() as usize,
            SizeBytes::Four  => self.deserialize::<u32>() as usize,
            SizeBytes::Eight => self.deserialize::<u64>() as usize,
        }
    }
    pub fn deserialize<T: Deserialize>(&mut self) -> T { T::deserialize(self) }
    pub fn deserialize_sized<T: SizedDeserialize>(&mut self, sb: SizeBytes) -> T {
        let size = self.read_size(sb);
        T::deserialize(self, size)
    }
    pub fn revision(&self) -> u32 { self.revision }
    pub fn finish(self) -> R { self.r }
}