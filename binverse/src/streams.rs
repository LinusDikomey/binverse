use std::io::{Read, Write};

use crate::{error::{BinverseError, BinverseResult}, serialize::{Deserialize, Serialize, SizeBytes, SizedDeserialize, SizedSerialize}, varint};

/// A serializer used to write serialized data to the writer.
pub struct Serializer<W: Write> {
    pub(crate) w: W,
}
impl<W: Write> Serializer<W> {
    /// Creates a new serializer. The revision will be written to the data to
    /// make it possible to parse the data in future revisions. If the revision
    ///  should not be written, use [`Serializer::new_no_revision`].
    pub fn new(w: W, revision: u32) -> BinverseResult<Self> {
        let mut s = Self { w };
        revision.serialize(&mut s)?;
        Ok(s)
    }

    /// Create a new Serializer, but without writing the revision into the stream.
    pub fn new_no_revision(w: W) -> Self {
        Self { w }
    }

    /// Write a raw byte buffer into the output. Should only be used when the
    /// size will be known when deserializing, use the [SizedSerialize]/[SizedDeserialize]
    /// implementations for `[u8]` or `Vec<u8>` otherwise.
    pub fn write(&mut self, buf: &[u8]) -> BinverseResult<()> {
        self.w.write_all(buf)?;
        Ok(())
    }

    pub(crate) fn write_size(&mut self, sb: SizeBytes, size: usize) -> BinverseResult<()> {
        use SizeBytes::*;
        let max_size = match sb {
            One         =>  u8::MAX as usize,
            Two         => u16::MAX as usize,
            Four        => u32::MAX as usize,
            Eight | Var => u64::MAX as usize,   
        };
        if size > max_size {
            return Err(BinverseError::SizeExceeded { limit: sb, found: size });
        }
        match sb {
            SizeBytes::One   => (size as  u8).serialize(self),
            SizeBytes::Two   => (size as u16).serialize(self),
            SizeBytes::Four  => (size as u32).serialize(self),
            SizeBytes::Eight => (size as u64).serialize(self),
            SizeBytes::Var   => varint::write(size as u64, &mut self.w)
        }
    }

    /// Serialize a sized data structure. Use the `size_bytes` parameter to
    /// control how many bytes are used to serialize the size of the data structure.
    /// An error will be returned when the size doesn't fit into the amount of bytes provided.
    /// For example, serializing a [Vec] with 258 elements will fail when using [SizeBytes::One].
    pub fn serialize_sized<T: SizedSerialize<W>>(&mut self, size_bytes: SizeBytes, t: &T) -> BinverseResult<()> {
        let size = t.size();
        self.write_size(size_bytes, size)?;
        t.serialize_sized(self, size)
    }
    /// Returns the inner writer.
    pub fn finish(self) -> W { self.w }
}

/// Reads previously serialized data from a reader. Note that all calls must be
/// the opposite from the calls used when serializing so the data matches.
pub struct Deserializer<R: Read> {
    pub(crate) r: R,
    revision: u32
}

impl<R: Read> Deserializer<R> {
    /// Creates a new deserializer from an underlying reader. The revision
    /// is read from the reader. If the revision should not be read, use
    /// [`Deserializer::new_no_revision`].
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
    /// revision than specified can lead to invalid data or errors.
    pub fn new_no_revision(r: R, revision: u32) -> Self {
        Self { r, revision }
    }
    
    /// Reads raw bytes into a byte slice. Should only be used when
    /// implementing new [Deserialize] implementations that can't make use of
    /// existing implementations.
    pub fn read(&mut self, buf: &mut [u8]) -> BinverseResult<()> {
        self.r.read_exact(buf)?;
        Ok(())
    }

    pub(crate) fn read_size(&mut self, sb: SizeBytes) -> BinverseResult<usize>  {
        Ok(match sb {
            SizeBytes::One   => self.deserialize::< u8>()? as usize,
            SizeBytes::Two   => self.deserialize::<u16>()? as usize,
            SizeBytes::Four  => self.deserialize::<u32>()? as usize,
            SizeBytes::Eight => self.deserialize::<u64>()? as usize,
            SizeBytes::Var   => varint::read(&mut self.r)? as usize
        })
    }

    /// Deserializes something. The type has to be known and has to match the
    /// type that was serialized previously.
    pub fn deserialize<T: Deserialize<R>>(&mut self) -> BinverseResult<T> { T::deserialize(self) }

    /// Deserializes a data structure with a size. Type and size_bytes have to
    /// match the serialized data structure.
    pub fn deserialize_sized<T: SizedDeserialize<R>>(&mut self, size_bytes: SizeBytes) -> BinverseResult<T> {
        let size = self.read_size(size_bytes)?;
        T::deserialize_sized(self, size)
    }
    /// Get the revision of the data currently being deserialized. Used when
    /// reading version-dependent data. 
    pub fn revision(&self) -> u32 { self.revision }
    /// Returns the inner reader.
    pub fn finish(self) -> R { self.r }
}