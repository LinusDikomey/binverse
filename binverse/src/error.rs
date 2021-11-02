use crate::serialize::SizeBytes;


#[derive(Debug)]
pub enum BinverseError {
    IO(std::io::Error),
    VarIntOverflow,
    InvalidUTF8,
    SizeExceeded {
        limit: SizeBytes, 
        found: usize
    },
    InvalidData
}

pub type BinverseResult<T> = Result<T, BinverseError>;

impl From<std::io::Error> for BinverseError {
    #[cfg_attr(feature = "inline", inline)]
    fn from(e: std::io::Error) -> BinverseError {
        BinverseError::IO(e)
    }
}