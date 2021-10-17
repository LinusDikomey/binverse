use std::{io::{Read, Write}};

use crate::{error::BinverseResult, serialize::{Deserialize, Serialize}, streams::{Deserializer, Serializer}};


// u8 / i8

impl Serialize for u8 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[*self])
    }
}
impl Deserialize for u8 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut b = [0; 1];
        d.read(&mut b)?;
        Ok(b[0])
    }
}

impl Serialize for i8 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        (*self as u8).serialize(s)
    }
}
impl Deserialize for i8 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut b = [0; 1];
        d.read(&mut b)?;
        Ok(b[0] as i8)
    }
}


// u16 / i16

impl Serialize for u16 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[
            (*self >> 8 & 255) as u8,
            (*self >> 0 & 255) as u8
        ])
    }
}
impl Deserialize for u16 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut b = [0; 2];
        d.read(&mut b)?;
        Ok(
            ((b[0] as u16) << 8) +
            ((b[1] as u16) << 0)
        )
    }
}

impl Serialize for i16 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        (*self as u16).serialize(s)
    }
}
impl Deserialize for i16 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(d.deserialize::<u16>()? as i16)
    }
}


// u32 / i32

impl Serialize for u32 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[
            (*self >> 24 & 255) as u8,
            (*self >> 16 & 255) as u8,
            (*self >>  8 & 255) as u8,
            (*self >>  0 & 255) as u8
        ])
    }
}
impl Deserialize for u32 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut b = [0; 4];
        d.read(&mut b)?;
        Ok(
            ((b[0] as u32) << 24) +
            ((b[1] as u32) << 16) +
            ((b[2] as u32) <<  8) +
            ((b[3] as u32) <<  0)
        )
    }
}

impl Serialize for i32 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        (*self as u32).serialize(s)
    }
}
impl Deserialize for i32 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(d.deserialize::<u32>()? as i32)
    }
}

// u64 / i64

impl Serialize for u64 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[
            (*self >> 56 & 255) as u8,
            (*self >> 48 & 255) as u8,
            (*self >> 40 & 255) as u8,
            (*self >> 32 & 255) as u8,
            (*self >> 24 & 255) as u8,
            (*self >> 16 & 255) as u8,
            (*self >>  8 & 255) as u8,
            (*self >>  0 & 255) as u8
        ])
    }
}
impl Deserialize for u64 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut b = [0; 8];
        d.read(&mut b)?;
        Ok(
            ((b[0] as u64) << 56) +
            ((b[1] as u64) << 48) +
            ((b[2] as u64) << 40) +
            ((b[3] as u64) << 32) +
            ((b[4] as u64) << 24) +
            ((b[5] as u64) << 16) +
            ((b[6] as u64) <<  8) +
            ((b[7] as u64) <<  0)
        )
    }
}

impl Serialize for i64 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        (*self as u64).serialize(s)
    }
}
impl Deserialize for i64 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(d.deserialize::<u64>()? as i64)
    }
}

// u128 / i128

impl Serialize for u128 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[
            (*self >> 120 & 255) as u8,
            (*self >> 112 & 255) as u8,
            (*self >> 104 & 255) as u8,
            (*self >>  96 & 255) as u8,
            (*self >>  88 & 255) as u8,
            (*self >>  80 & 255) as u8,
            (*self >>  72 & 255) as u8,
            (*self >>  64 & 255) as u8,

            (*self >> 56 & 255) as u8,
            (*self >> 48 & 255) as u8,
            (*self >> 40 & 255) as u8,
            (*self >> 32 & 255) as u8,
            (*self >> 24 & 255) as u8,
            (*self >> 16 & 255) as u8,
            (*self >>  8 & 255) as u8,
            (*self >>  0 & 255) as u8
        ])
    }
}
impl Deserialize for u128 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut b = [0; 16];
        d.read(&mut b)?;

        Ok(
            ((b[0] as u128) << 120) +
            ((b[1] as u128) << 112) +
            ((b[2] as u128) << 104) +
            ((b[3] as u128) <<  96) +
            ((b[4] as u128) <<  88) +
            ((b[5] as u128) <<  80) +
            ((b[6] as u128) <<  72) +
            ((b[7] as u128) <<  64) +

            ((b[ 8] as u128) << 56) +
            ((b[ 9] as u128) << 48) +
            ((b[10] as u128) << 40) +
            ((b[11] as u128) << 32) +
            ((b[12] as u128) << 24) +
            ((b[13] as u128) << 16) +
            ((b[14] as u128) <<  8) +
            ((b[15] as u128) <<  0)
        )
    }
}
impl Serialize for i128 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        (*self as u128).serialize(s)
    }
}
impl Deserialize for i128 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(d.deserialize::<u128>()? as i128)
    }
}

// f32
impl Serialize for f32 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        unsafe { std::mem::transmute::<_, u32>(*self) }.serialize(s)
    }
}
impl Deserialize for f32 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(unsafe { std::mem::transmute::<u32, _>(d.deserialize()?) })
    }
}

// f64
impl Serialize for f64 {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        unsafe { std::mem::transmute::<_, u64>(*self) }.serialize(s)
    }
}
impl Deserialize for f64 {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(unsafe { std::mem::transmute::<u64, _>(d.deserialize()?) })
    }
}