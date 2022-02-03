use crate::serialize::SizeBytes;
use std::fmt;

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

impl fmt::Display for BinverseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "Binverse IO Error: {}", err),
            Self::VarIntOverflow => write!(f, "VarInt overflow occured"),
            Self::InvalidUTF8 => write!(f, "Invalid UTF8 data encountered"),
            Self::SizeExceeded { limit, found } => write!(f, "Data structure size was exceeded, maximum allowed length was {} ({:?}) but found {}", limit.maximum(), limit, found),
            Self::InvalidData => write!(f, "Data was invalid")
        }
    }
}

impl From<std::io::Error> for BinverseError {
    /// Wraps an IO error in the IO variant of BinverseError.
    fn from(e: std::io::Error) -> BinverseError {
        BinverseError::IO(e)
    }
}

impl std::error::Error for BinverseError { }