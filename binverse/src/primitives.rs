use std::io::{Read, Write};

use crate::{error::{BinverseError, BinverseResult}, serialize::{Deserialize, Serialize, SizeBytes, SizedDeserialize, SizedSerialize}, streams::{Deserializer, Serializer}};

impl<W: Write> Serialize<W> for bool {
    fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.write(&[*self as u8])
    }
}
impl<R: Read> Deserialize<R> for bool {
    fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
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
            impl<W: Write> Serialize<W> for $t {
                fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
                    s.write(&self.to_le_bytes())
                }
            }
            impl<R: Read> Deserialize<R> for $t {
                fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
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

impl<W: Write, T, const N: usize> Serialize<W> for [T; N]
where T: Serialize<W> {
    fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
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
            impl<R: Read, T: Deserialize<R>> Deserialize<R> for [T; $n] {
                fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
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

/*impl<W: Write, T: SizedDeserialize<W>> Serialize<W> for T {
    fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        s.serialize_sized(SizeBytes::Var, self)
    }
}
impl<R: Read, T: SizedDeserialize<R>> Deserialize<R> for T {
    fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        d.deserialize_sized(SizeBytes::Var)   
    }
}*/

// str/String
impl<W: Write> SizedSerialize<W> for &str {
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        s.write(self[..size].as_bytes())
    }
    fn size(&self) -> usize {
        self.len()
    }
}

macro_rules! ser_sized {
    ($({$t: ty [$($generic: tt),*] [$([$($tree: tt)*]),*]})*) => {
        $(
            impl<W: Write, $($generic),*> Serialize<W> for $t 
            where $($($tree)*)* {
                fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
                    s.serialize_sized(SizeBytes::Var, self)
                }
            }
        )*
    }
}
macro_rules! deser_sized {
    ($({$t: ty [$($generic: tt),*] [$([$($tree: tt)*]),*]})*) => {
        $(
            impl<R: Read, $($generic),*> Deserialize<R> for $t 
            where $($($tree)*)* {
                fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
                    d.deserialize_sized(SizeBytes::Var)
                }
            }
        )*
    }
}

ser_sized!{ {&str [][]} {String [][]} {&[T] [T] [[T: Serialize<W>]]} {Vec<T> [T] [[T: Serialize<W>]]} }
deser_sized!{ {String [][]} {Vec<T> [T] [[T: Deserialize<R>]]} }

impl<W: Write> SizedSerialize<W> for String {
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        // TODO: slicing here could make code more inefficient when the size is usually correct
        s.write(self[..size].as_bytes())
    }
    fn size(&self) -> usize { self.len() }
}
impl<R: Read> SizedDeserialize<R> for String {
    fn deserialize_sized(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self> {
        let mut b = vec![0; size];
        d.read(&mut b)?;
        String::from_utf8(b).or(Err(BinverseError::InvalidUTF8))
    }
}

impl<W: Write, T> SizedSerialize<W> for &[T]
where T: Serialize<W> {
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        for elem in &self[0..size] {
            elem.serialize(s)?;
        }
        Ok(())
    }
    fn size(&self) -> usize {
        self.len()
    }
}
impl<W: Write, T: Serialize<W>> SizedSerialize<W> for Vec<T> {
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        self.as_slice().serialize_sized(s, size)
    }
    fn size(&self) -> usize {
        self.len()
    }
}
impl<R: Read, T: Deserialize<R>> SizedDeserialize<R> for Vec<T> {
    fn deserialize_sized(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self> {
        (0..size).map(|_| d.deserialize()).collect::<BinverseResult<Vec<_>>>()
    }
}