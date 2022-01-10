use std::{io::{Read, Write}, collections::HashMap, hash::Hash, mem::ManuallyDrop};

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

/// To initialize an array element by element, we need an array containing
/// uninitialized data. InitializingArray does this but only drops the elements
/// that are initalized to prevent it from dropping invalid data if it is
/// dropped before it is fully initialized.
struct InitializingArray<T, const N: usize> {
    inner: ManuallyDrop<[T; N]>,
    initialized_to: usize
}
impl<T, const N: usize> InitializingArray<T, N> {
    fn new() -> Self {
        Self {
            inner: ManuallyDrop::new(unsafe { std::mem::MaybeUninit::uninit().assume_init() }),
            initialized_to: 0
        }
    }
    fn push(&mut self, t: T) {
        assert!(self.initialized_to < self.inner.len());
        self.inner[self.initialized_to] = t;
        self.initialized_to += 1;
    }
    fn get(self) -> [T; N] {
        assert_eq!(self.inner.len(), self.initialized_to);
        // SAFETY: we prevent the object from dropping by manually pulling out the fields.
        // Because we get all the fields, this is safe.
        // The array is also guaranteed to be fully initialized because of the assert above;
        unsafe {
            let x = ManuallyDrop::new(self);
            let inner = std::ptr::read(&(*x).inner);
            let _initialized_to = std::ptr::read(&(*x).initialized_to);
            ManuallyDrop::into_inner(inner)
        }
    }
}
impl<T, const N: usize> Drop for InitializingArray<T, N> {
    fn drop(&mut self) {
        // only the initialized data is dropped
        unsafe { std::ptr::drop_in_place(&mut self.inner[..self.initialized_to]) };
    }
}

impl<R: Read, T: Deserialize<R>, const N: usize> Deserialize<R> for [T; N] {
    fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        let mut init_arr = InitializingArray::new();
        for _ in 0..N {
            init_arr.push(d.deserialize()?);
        }
        Ok(init_arr.get())
    }
}

impl<W: Write, T: Serialize<W>> Serialize<W> for Option<T> {
    fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
        if let Some(e) = self {
            1_u8.serialize(s)?;
            e.serialize(s)?;
        } else {
            0_u8.serialize(s)?;
        }
        Ok(())
    }
}
impl<R: Read, T: Deserialize<R>> Deserialize<R> for Option<T> {
    fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
        Ok(match d.deserialize()? {
            0_u8 => None,
            1_u8 => Some(d.deserialize()?),
            _ => return Err(BinverseError::InvalidData)
        })
    }
}

// tuples
macro_rules! tuples {
    ($($($t: ident $elem: tt)*;)*) => {
        $(
            impl<W: Write, $($t: Serialize<W>),*> Serialize<W> for ($($t),*) {
                fn serialize(&self, s: &mut Serializer<W>) -> BinverseResult<()> {
                    $( self.$elem.serialize(s)?; )*
                    Ok(())
                }
            }
            impl<R: Read, $($t: Deserialize<R>),*> Deserialize<R> for ($($t),*) {
                fn deserialize(d: &mut Deserializer<R>) -> BinverseResult<Self> {
                    Ok(($( <$t as $crate::serialize::Deserialize<R>>::deserialize(d)?, )*))
                }
            }
        )*
    }
}

tuples! {
    A 0 B 1;
    A 0 B 1 C 2;
    A 0 B 1 C 2 D 3;
    A 0 B 1 C 2 D 3 E 4;
    A 0 B 1 C 2 D 3 E 4 F 5;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10 L 11;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10 L 11 M 11;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10 L 11 M 11 N 12;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10 L 11 M 11 N 12 O 13;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10 L 11 M 11 N 12 O 13 P 14;
    A 0 B 1 C 2 D 3 E 4 F 5 G 6 H 7 I 8 J 9 K 10 L 11 M 11 N 12 O 13 P 14 Q 15;
}

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

ser_sized!{ {&str [][]} {String [][]} {&[T] [T] [[T: Serialize<W>]]} {Vec<T> [T] [[T: Serialize<W>]]} {HashMap<K, V> [K, V] [[K: Serialize<W>, V: Serialize<W>]]} }
deser_sized!{ {String [][]} {Vec<T> [T] [[T: Deserialize<R>]]} {HashMap<K, V> [K, V] [[K: Deserialize<R> + Eq + Hash, V: Deserialize<R>]]} }

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

impl<W: Write, K: Serialize<W>, V: Serialize<W>> SizedSerialize<W> for HashMap<K, V> {
    fn serialize_sized(&self, s: &mut Serializer<W>, size: usize) -> BinverseResult<()> {
        for (k, v) in self.iter().take(size) {
            k.serialize(s)?;
            v.serialize(s)?;
        }
        Ok(())
    }
    fn size(&self) -> usize {
        self.len()
    }
}
impl<R: Read, K: Deserialize<R> + Eq + Hash, V: Deserialize<R>> SizedDeserialize<R> for HashMap<K, V> {
    fn deserialize_sized(d: &mut Deserializer<R>, size: usize) -> BinverseResult<Self> {
        Ok((0..size).map(|_| Ok((d.deserialize()?, d.deserialize()?))).collect::<BinverseResult<HashMap<K, V>>>()?)
    }
}