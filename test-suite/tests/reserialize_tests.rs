use std::{fmt::Debug, io::Cursor, collections::HashMap};

use binverse::{serialize::{Deserialize, Serialize, SizeBytes, SizedSerialize, SizedDeserialize}, streams::{Deserializer, Serializer}, error::BinverseError};
use binverse_derive::serializable;

fn reserialize_test<T : Serialize + Deserialize + PartialEq + Debug>(val: T) {
    let mut s = Serializer::new(Vec::new(), 0).unwrap();
    val.serialize(&mut s).unwrap();
    let buf = s.finish();
    let mut d = Deserializer::new(Cursor::new(buf)).unwrap();
    let new_val: T = d.deserialize().unwrap();
    assert_eq!(val, new_val, "{}", std::any::type_name::<T>());
    let finished = d.finish();
    let pos = finished.position();
    assert_eq!(finished.into_inner().len() as u64 - pos, 0, "leftover bytes after deserialing");
}

fn reserialize_sized_test<'a, T : SizedSerialize + SizedDeserialize + PartialEq + Debug>(val: T, sb: SizeBytes) {
    let mut s = Serializer::new(Vec::new(), 0).unwrap();
    s.serialize_sized(sb, &val).unwrap();
    let buf = s.finish();
    let mut d = Deserializer::new(Cursor::new(buf)).unwrap();
    let new_val: T = d.deserialize_sized(sb).unwrap();
    assert_eq!(val, new_val, "{}", std::any::type_name::<T>());
    let finished = d.finish();
    let pos = finished.position();
    assert_eq!(finished.into_inner().len() as u64 - pos, 0, "leftover bytes after deserialing");
}

fn test_all<'a, T>(vals: &[T]) 
where T : Serialize + Deserialize + PartialEq + Debug + Clone {
    for val in vals {
        reserialize_test(val.clone());
    }
}

#[test]
fn primitive_serialization() {
    use binverse::error::BinverseError;

    

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
    

    test_all(&[
        Some(Some(-3.0)),
        Some(Some(5.0)),
        Some(Some(f64::INFINITY)),
        Some(None),
        None,
    ]);

    test_all(&[
        Some("Option string".to_owned()),
        Some(String::new()),
        None
    ]);

    reserialize_test([1, 2, 3]);
    reserialize_test::<[i32; 0]>([]);
    reserialize_test(["".to_owned(), "ABCD".to_owned()]);
    
    let string = "A random example string";
    reserialize_sized_test(string.to_owned(), SizeBytes::One);
    reserialize_sized_test(string.to_owned(), SizeBytes::Two);
    reserialize_sized_test(string.to_owned(), SizeBytes::Four);
    reserialize_sized_test(string.to_owned(), SizeBytes::Eight);

    reserialize_sized_test("a".repeat(255), SizeBytes::One);

    let mut s = Serializer::new(Vec::new(), 0).unwrap();
    match s.serialize_sized(SizeBytes::One, &"a".repeat(256)).unwrap_err() {
        BinverseError::SizeExceeded { limit: SizeBytes::One, found: 256 } => (),
        err => panic!("Invalid error: {:?}", err)
    }

    let mut map = HashMap::new();
    map.insert("Hello".to_owned(), [1, 2, 3]);
    map.insert("This is a map".to_owned(), [4, 5, -3]);
    reserialize_sized_test(map, SizeBytes::One);
}

#[test]
fn array_drop() {
    let mut d = Deserializer::new_no_revision([1_u8, 0, 0, 1, 1].as_slice(), 0);
    assert_eq!(d.deserialize::<[Option<bool>; 3]>().unwrap(), [Some(false), None, Some(true)]);
    static mut DROPS: u32 = 0;

    // struct to count number of drops
    #[serializable]
    struct DropTest(bool);
    impl Drop for DropTest {
        fn drop(&mut self) {
            println!("drop");
            // should work for this test because it only runs on one thread
            unsafe { DROPS += 1; }
        }
    }
    let mut d = Deserializer::new_no_revision([0_u8, 1_u8, 1_u8, 0_u8, 1_u8, 2_u8].as_slice(), 0);
    assert_eq!(unsafe { DROPS }, 0);
    let first_2 = d.deserialize::<[DropTest; 2]>().unwrap();
    assert_eq!(unsafe { DROPS }, 0);
    matches!(&first_2, &[DropTest(false), DropTest(true)]);
    drop(first_2);
    assert_eq!(unsafe { DROPS }, 2);

    matches!(d.deserialize::<[DropTest; 4]>(), Err(BinverseError::InvalidData));
    // fails after reading 3 elements, drops them
    assert_eq!(unsafe { DROPS }, 5);
}


#[test]
fn structs() {
    #[serializable]
    #[derive(PartialEq, Debug, Clone, Default)]
    struct Vec3 {
        x: f32,
        y: f32,
        z: f32
    }

    test_all(&[
        Vec3 { x: 1354.124, y: -124.32, z: 124.12 },
        Vec3 { x: f32::MAX, y: 0.0, z: 0.0 }
    ]);

    #[serializable]
    #[derive(PartialEq, Debug, Clone)]
    struct Example1 {
        position: Vec3,
        name: String,
        alive: bool
    }
    test_all(&[
        Example1 {
            position: Vec3 { x: 123.4, y: -1.0, z: 5.4 },
            name: String::from("Player Entity"),
            alive: true
        },
        Example1 {
            position: Vec3 { x: 3543.4, y: 3.0, z: std::f32::consts::PI },
            name: String::from(format!("An entity with a very long name: {}", "VeryLongName".repeat(10000))),
            alive: false
        },
    ]);


    #[derive(Debug, Clone, PartialEq)]
    #[serializable]
    struct Test1;
    #[derive(Debug, Clone, PartialEq)]
    #[serializable]
    struct Test2(f32);
    
    reserialize_test(Test1);
    test_all(&[
        Test2(1252.135),
        Test2(-352.10),
        Test2(124.21),
        Test2(1294.65),
        Test2(f32::INFINITY)
    ]);
    

}
