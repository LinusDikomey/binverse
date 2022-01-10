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