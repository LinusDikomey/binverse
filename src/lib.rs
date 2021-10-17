pub mod streams;
pub mod serialize;
pub mod primitives;
pub mod varint;
pub mod error;


#[cfg(test)]
mod test {

    use std::{fmt::Debug, io::{Cursor, Seek}};
    use crate::{error::BinverseError, serialize::{SizeBytes, Serialize, Deserialize, SizedSerialize, SizedDeserialize}, streams::{Deserializer, Serializer}, varint};

    fn reserialize_test<T : Serialize + Deserialize + PartialEq + Debug>(val: T, name: &str) {
        let mut s = Serializer::new(Vec::new(), 0).unwrap();
        val.serialize(&mut s).unwrap();
        let buf = s.finish();
        let mut d = Deserializer::new(buf.as_slice()).unwrap();
        let new_val: T = d.deserialize().unwrap();
        assert_eq!(val, new_val, "{}", name);
    }

    fn reserialize_sized_test<T : SizedSerialize + SizedDeserialize + PartialEq + Debug>(val: T, sb: SizeBytes, name: &str) {
        let mut s = Serializer::new(Vec::new(), 0).unwrap();
        s.serialize_sized(sb, &val).unwrap();
        let buf = s.finish();
        let mut d = Deserializer::new(buf.as_slice()).unwrap();
        let new_val: T = d.deserialize_sized(sb).unwrap();
        assert_eq!(val, new_val, "{}", name);
    }

    fn test_all<T>(vals: &[T]) 
    where T : Serialize + Deserialize + PartialEq + Debug + Clone {
        for val in vals {
            reserialize_test(val.clone(), std::any::type_name::<T>());
        }
    }

    #[test]
    fn primitive_serialization() {
        test_all(&(0..=u8::MAX).collect::<Vec<_>>());
        
        test_all(&(0..=u16::MAX).step_by(3).collect::<Vec<_>>());
        test_all(&[0, u16::MAX, u16::MAX - 1]);

        test_all(&[
            0,
            1,
            0xFF_AB_CD_EF,
            0x12_34_56_78,
            0xFF_AB_CD_EF,
            0x12_AB_CD_EF,
            u32::MAX,
            u32::MAX - 1,
        ]);

        test_all(&[
            0,
            1,
            0xFF_AB_CD_EF_12_34_56_78,
            0x12_34_56_78_00_FF_AA_BB,
            0x00_00_FF_AA_FF_AB_CD_EF,
            0x12_AB_CD_EF_34_56_78_00,
            u64::MAX,
            u64::MAX - 1,
        ]);

        test_all(&[
            0,
            1,
            0xFF_AB_CD_EF_12_34_56_78_90_36_31_57_12_68_26_18,
            0x12_34_56_78_00_FF_AA_BB_98_76_54_32_10_16_23_63,
            u128::MAX,
            u128::MAX - 1,
        ]);
        
        
        let string = "A random example string";
        reserialize_sized_test(string.to_owned(), SizeBytes::One, "String");
        reserialize_sized_test(string.to_owned(), SizeBytes::Two, "String");
        reserialize_sized_test(string.to_owned(), SizeBytes::Four, "String");
        reserialize_sized_test(string.to_owned(), SizeBytes::Eight, "String");

        reserialize_sized_test("a".repeat(255), SizeBytes::One, "String");

        let mut s = Serializer::new(Vec::new(), 0).unwrap();
        match s.serialize_sized(SizeBytes::One, &"a".repeat(256)).unwrap_err() {
            BinverseError::SizeExceeded { limit: SizeBytes::One, found: 256 } => (),
            err => panic!("Invalid error: {:?}", err)
        }
    }

    #[test]
    fn varints() {
        fn test_varint(x: u64) {
            let mut c = Cursor::new(vec![0; 11]);
            varint::write_varint(x, &mut c).unwrap();
            c.rewind().unwrap();
            let x2 = varint::read_varint(&mut c).unwrap_or_else(|e| panic!("Got error while testing varints: {:?}, with value: {}, bytes: {:?}", e, x, c.clone().into_inner()));
            if x != x2 {
                panic!("Mismatched values while testing varints with value: {} != {}, bytes: {:?}", x, x2, c.into_inner())
            }
        }

        for x in [
            0,
            1,
            0xFF_AB_CD_EF_12_34_56_78,
            0x12_34_56_78_00_FF_AA_BB,
            0x00_00_FF_AA_FF_AB_CD_EF,
            0x12_AB_CD_EF_34_56_78_00,
            u64::MAX,
            u64::MAX - 1,
        ] {
            eprintln!("Testing varint {}", x);
            test_varint(x);
        }
    }
}