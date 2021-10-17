use std::{io::{Read, Write}};

use crate::{error::BinverseResult, serialize::{Deserialize, Serialize}, streams::{Deserializer, Serializer}};


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