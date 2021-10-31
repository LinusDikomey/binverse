use std::io::{Read, Write};

use crate::{error::{RenameSymbol, BinverseResult}, serialize::{Deserialize, Serialize, SizeBytes, SizedDeserialize, SizedSerialize}, varint};

pub struct Serializer<W: Write> {
    pub(crate) w: W,
}
impl<W: Write> Serializer<W> {
    pub fn new(w: W, revision: u32) -> BinverseResult<Self> {
        let mut s = Self { w };
        revision.serialize(&mut s)?;
        Ok(s)
    }

    /// Create a new Serializer, but without writing the revision into the stream
    pub fn new_no_revision(w: W) -> Self {
        Self { w }
    }

    pub fn write(&mut self, buf: &[u8]) -> BinverseResult<()> {
        self.w.write_all(buf)?;
        Ok(())
    }
    pub fn write_size(&mut self, sb: SizeBytes, size: usize) -> BinverseResult<()> {
        use SizeBytes::*;
        let max_size = match sb {
            One         =>  u8::MAX as usize,
            Two         => u16::MAX as usize,
            Four        => u32::MAX as usize,
            Eight | Var => u64::MAX as usize,   
        };
        if size > max_size {
            return Err(RenameSymbol::SizeExceeded { limit: sb, found: size });
        }
        match sb {
            SizeBytes::One   => (size as  u8).serialize(self),
            SizeBytes::Two   => (size as u16).serialize(self),
            SizeBytes::Four  => (size as u32).serialize(self),
            SizeBytes::Eight => (size as u64).serialize(self),
            SizeBytes::Var   => varint::write(size as u64, &mut self.w)
        }
    }
    pub fn serialize_sized<T: SizedSerialize>(&mut self, sb: SizeBytes, t: &T) -> BinverseResult<()> {
        let size = t.size();
        self.write_size(sb, size)?;
        t.serialize(self, size)
    }
    //pub fn revision(&self) -> u32 { self.revision }
    pub fn finish(self) -> W { self.w }
}

pub struct Deserializer<R: Read> {
    pub(crate) r: R,
    revision: u32
}

impl<R: Read> Deserializer<R> {
    pub fn new(r: R) -> BinverseResult<Self> {
        let mut d = Self {
            r,
            revision: 0
        };
        d.revision = d.deserialize()?;
        Ok(d)
    }

    /// Create a new Deserializer, but without reading the revision from the stream.
    /// Instead, the revision has to be passed. Providing data created in a different
    /// revision than specified can lead to invalid data or errors
    pub fn new_no_revision(r: R, revision: u32) -> Self {
        Self { r, revision }
    }
    
    pub fn read(&mut self, buf: &mut [u8]) -> BinverseResult<()> {
        self.r.read_exact(buf)?;
        Ok(())
    }
    pub fn read_size(&mut self, sb: SizeBytes) -> BinverseResult<usize>  {
        Ok(match sb {
            SizeBytes::One   => self.deserialize::< u8>()? as usize,
            SizeBytes::Two   => self.deserialize::<u16>()? as usize,
            SizeBytes::Four  => self.deserialize::<u32>()? as usize,
            SizeBytes::Eight => self.deserialize::<u64>()? as usize,
            SizeBytes::Var   => varint::read(&mut self.r)? as usize
        })
    }
    pub fn deserialize<T: Deserialize>(&mut self) -> BinverseResult<T> { T::deserialize(self) }
    pub fn deserialize_sized<T: SizedDeserialize>(&mut self, sb: SizeBytes) -> BinverseResult<T> {
        let size = self.read_size(sb)?;
        T::deserialize(self, size)
    }
    pub fn revision(&self) -> u32 { self.revision }
    pub fn finish(self) -> R { self.r }
}