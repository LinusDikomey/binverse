use std::io::{Read, Write};
use crate::error::{BinverseResult, RenameSymbol};

/// The maximum length in bytes of a varint (u64)
pub const MAX_LEN: usize = 10;

/// Reads an unsigned 64-bit varint number from a Reader
pub fn read<R: Read>(r: &mut R) -> BinverseResult<u64> {
    let mut x: u64 = 0;
    let mut s = 0;
    let mut b = [0_u8; 1];
    for i in 0..MAX_LEN {
        r.read_exact(&mut b)?;
        let b = b[0];
        if b < 0x80 {
            if i == MAX_LEN-1 && b > 1 {
                return Err(RenameSymbol::VarIntOverflow)
            }
            return Ok(x | (b as u64) << s)
        }
        x |= ((b&0x7f) as u64) << s;
        s += 7;
    }
    // varint was too long and can be considered invalid
    Err(RenameSymbol::VarIntOverflow)
}

/// Writes an unsigned 64-bit varint number to a Writer
pub fn write<W: Write>(mut x: u64, w: &mut W) -> Result<(), RenameSymbol> {
    while x >= 0x80 {
        w.write_all(&[x as u8 | 0x80])?;
        x >>= 7;
    }
    w.write_all(&[x as u8])?;
    Ok(())
}