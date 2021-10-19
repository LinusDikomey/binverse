use std::{io::{Read, Write}};

use crate::{error::{BinverseError, BinverseResult}, serialize::{Deserialize, Serialize, SizeBytes, SizedDeserialize, SizedSerialize}, streams::{Deserializer, Serializer}};

impl Serialize for bool {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[*self as u8])
    }
}
impl Deserialize for bool {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut buf = [0];
        d.read(&mut buf)?;
        match buf[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BinverseError::InvalidData)
        }
    }
}


macro_rules! number_impls {
    ($($t: ty, $bytes: expr),*) => {
        $(
            impl Serialize for $t {
                fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
                    s.write(&self.to_le_bytes())
                }
            }
            impl Deserialize for $t {
                fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
                    let mut b = [0; $bytes];
                    d.read(&mut b)?;
                    Ok(<$t>::from_le_bytes(b))
                }
            }
        )*
    };
}
number_impls!(
    u8, 1, i8, 1,
    u16, 2, i16, 2,
    u32, 4, i32, 4,
    u64, 8, i64, 8,
    u128, 16, i128, 16,
    f32, 4,
    f64, 8
);

impl<T, const N: usize> Serialize for [T; N]
where T: Serialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        for elem in self {
            elem.serialize(s)?;
        }
        Ok(())
    }
}

// this is pretty stupid but rust has no good way to handle array initialization with varying sizes

macro_rules! array_deserialize {
    ($($n: expr, $($i: ident)*);*) => {
        $(
            impl<T> Deserialize for [T; $n]
            where T: Deserialize {
                fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
                    $(
                        let $i: T = d.deserialize()?;
                    )*
                    Ok([
                        $(
                            $i
                        ),*
                    ])

                }
            }
        )*
    };
}

array_deserialize! {
    1,  e0;
    2,  e0 e1;
    3,  e0 e1 e2;
    4,  e0 e1 e2 e3;
    5,  e0 e1 e2 e3 e4;
    6,  e0 e1 e2 e3 e4 e5;
    7,  e0 e1 e2 e3 e4 e5 e6;
    8,  e0 e1 e2 e3 e4 e5 e6 e7;
    9,  e0 e1 e2 e3 e4 e5 e6 e7 e8;
    10, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9;
    11, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10;
    12, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11;
    13, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12;
    14, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13;
    15, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14;
    16, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15;
    17, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16;
    18, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17;
    19, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18;
    20, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19;
    21, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20;
    22, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21;
    23, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22;
    24, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23;
    25, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24;
    26, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25;
    27, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25 e26;
    28, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25 e26 e27;
    29, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25 e26 e27 e28;
    30, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25 e26 e27 e28 e29;
    31, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25 e26 e27 e28 e29 e30;
    32, e0 e1 e2 e3 e4 e5 e6 e7 e8 e9 e10 e11 e12 e13 e14 e15 e16 e17 e18 e19 e20 e21 e22 e23 e24 e25 e26 e27 e28 e29 e30 e31
}

// implement Serialize/Deserialize with a default size of 'SizeBytes::Var' for 
// anything implementing SizedSerialize/SizedSerialize:

impl<T> Serialize for T
where T: SizedSerialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.serialize_sized(SizeBytes::Var, self)
    }
}
impl<T> Deserialize for T
where T: SizedDeserialize {
    fn deserialize<R: Read>(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        d.deserialize_sized(SizeBytes::Var)   
    }
}

// str/String
impl SizedSerialize for str {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        s.write(self[..size].as_bytes())
    }
    fn size(&self) -> usize {
        self.len()
    }
}

impl SizedSerialize for String {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        // TODO: slicing here could make code more inefficient when the size is usually correct
        s.write(self[..size].as_bytes())
    }
    fn size(&self) -> usize { self.len() }
}
impl SizedDeserialize for String {
    fn deserialize<R: Read>(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self> {
        let mut b = vec![0; size];
        d.read(&mut b)?;
        String::from_utf8(b).or(Err(BinverseError::InvalidUTF8))
    }
}

impl<T> SizedSerialize for &[T]
where T: Serialize {
    fn serialize<W: Write>(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        for elem in &self[0..size] {
            elem.serialize(s)?;
        }
        Ok(())
    }
    fn size(&self) -> usize {
        self.len()
    }
}
impl<T> SizedDeserialize for Vec<T>
where T: Deserialize {
    fn deserialize<R: Read>(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self> {
        (0..size).map(|_| d.deserialize()).collect::<Result<Vec<_>, _>>()
    }
}