use crate::serialize::SizeBytes;


#[derive(Debug)]
pub enum RenameSymbol {
    IO(std::io::Error),
    VarIntOverflow,
    InvalidUTF8,
    SizeExceeded {
        limit: SizeBytes, 
        found: usize
    },
    InvalidData
}

pub type BinverseResult<T> = Result<T, RenameSymbol>;

impl From<std::io::Error> for RenameSymbol {
    fn from(e: std::io::Error) -> RenameSymbol {
        RenameSymbol::IO(e)
    }
}