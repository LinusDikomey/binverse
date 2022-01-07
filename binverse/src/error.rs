use crate::serialize::SizeBytes;

/// An error suggesting something went wrong during data (de)serialization.
/// It can occur due to invald data, an IO error or an error in the
/// Serialize or Deserialize implementation.
#[derive(Debug)]
pub enum BinverseError {
    /// An io error originating from the underlying data stream.
    IO(std::io::Error),
    /// A VarInt was larger than expected suggesting invalid data.
    VarIntOverflow,
    /// A UTF8 text sequence contained invalid characters.
    InvalidUTF8,
    /// A variable sized data structure with limited size bytes was too large
    /// for the SizeBytes limitation.
    SizeExceeded {
        /// The SizeBytes limit set for the data structure.
        limit: SizeBytes, 
        /// The number of elements found in the data structure.
        found: usize
    },
    /// A generic invalid data error 
    InvalidData
}
/// A type alias for a Result with a BinverseError as the error type.
pub type BinverseResult<T> = Result<T, BinverseError>;

impl From<std::io::Error> for BinverseError {
    #[cfg_attr(feature = "inline", inline)]
    /// Wraps an IO error in the IO variant of BinverseError.
    fn from(e: std::io::Error) -> BinverseError {
        BinverseError::IO(e)
    }
}