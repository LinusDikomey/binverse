//! BinVerSe (Binary Versioned Serialization) provides fast, compact and simple
//! serialization of Rust data structures. Data is simply written to a binary
//! stream without any additional information. Backwards compatibility is
//! ensured through a global data revision number. With the binverse_derive
//! crate, the [`binverse_derive::serializable`] attribute macro automatically
//! implements the [`Serialize`]/[`Deserialize`] traits
//! 
//! [`Serialize`]: [serialize::Serialize]
//! [`Deserialize`]: [serialize::Deserialize]

#![warn(missing_docs)]

/// Serialize/Deserialize traits as well as sized versions of the traits.
pub mod serialize;
/// Provides Serializer/Deserializer types for reading and writing data.
pub mod streams;
/// Serialize/Deserialize implemnentations for primitive types.
pub mod primitives;
/// Variable sized integer read/write functions.
pub mod varint;
/// BinverseError as well as a BinverseResult type alias.
pub mod error;

pub use binverse_derive::serializable;

/// Writes a single object to a writer. When writing multiple objects, use [Serializer](streams::Serializer) instead.
/// The revision is also written to the writer for data backwards compatiblity.
/// 
/// This is the counterpart to [read()].
pub fn write<T: serialize::Serialize<W>, W: std::io::Write>(w: W, object: T, current_revision: u32) -> error::BinverseResult<W> {
    let mut s = streams::Serializer::new(w.into(), current_revision)?;
    object.serialize(&mut s)?;
    Ok(s.finish())
}

/// Reads a single object from a reader. When reading multiple objects, use [Deserializer](streams::Deserializer) instead.
/// The revision is also read from the reader so old data can be read.
/// 
/// This is the counterpart to [write()].
pub fn read<R: std::io::Read, T: serialize::Deserialize<R>>(r: R) -> error::BinverseResult<(T, R)> {
    let mut d = streams::Deserializer::new(r.into())?;
    let t = d.deserialize()?;
    Ok((t, d.finish()))
}


/// Writes a single object to a writer without writing the revision. When writing multiple objects, use
/// [Serializer](streams::Serializer) instead. If you want to be able to parse data in future versions,
/// use the regular [write()] function.
/// This can be used when the data won't change in the future or the revision can be implied from context when reading.
/// 
/// This is the counterpart to [read_no_revision()].
pub fn write_no_revision<T: serialize::Serialize<W>, W: std::io::Write>(w: W, object: T) -> error::BinverseResult<W> {
    let mut s = streams::Serializer::new_no_revision(w.into());
    object.serialize(&mut s)?;
    Ok(s.finish())
}

/// Reads a single object from a reader without reading a revision. When reading multiple objects, use
/// [Deserializer](streams::Deserializer) instead. The `revision` has to be supplied as a parameter.
/// This can be used when the data won't change in the future or the revision can be implied from context when reading.
/// This is the counterpart to [write_no_revision].
pub fn read_no_revision<R: std::io::Read, T: serialize::Deserialize<R>>(r: R, revision: u32) -> error::BinverseResult<(T, R)> {
    let mut d = streams::Deserializer::new_no_revision(r.into(), revision);
    let t = d.deserialize()?;
    Ok((t, d.finish()))
}
