use std::fmt::Debug;

use binverse::{serialize::Serialize, streams::{Deserializer, Serializer}};
use binverse_derive::serializable;

#[test]
fn basic_struct() {
    #[serializable]
    #[derive(Debug, PartialEq)]
    struct Example {
        a: i32,
        b: f32,
        c: String
    }

    let example = Example {
        a: -1253891,
        b: 44223.125,
        c: String::from("Hello binverse!")
    };

    let mut serializer = Serializer::new(Vec::new(), 0).unwrap();
    example.serialize(&mut serializer).unwrap();
    let data = serializer.finish();

    assert_eq!(data.len(), 
        4 + // revision (version of the data) 
        4 + // a: i32       
        4 + // b: f32
        1 + // length of the following string (VarInt)
        // the bytes of the string:
        "Hello binverse!".len()
    );

    let mut deserializer = Deserializer::new(data.as_slice()).unwrap();
    let example_deserialized: Example = deserializer.deserialize().unwrap();
    assert_eq!(example, example_deserialized);
    
    assert_eq!(deserializer.finish().len(), 0, "Remaining bytes after deserializing");
}

#[test]
fn simple_macro_attribs() {
    // struct with named fields
    #[serializable]
    struct Test1 {
        a: Added<3, i32>,
        b: Removed<2, i32>,
        c: SizeBytes<8, String>
    }

    // tuple struct
    #[serializable]
    struct Test2 (
        Added<3, i32>,
        Removed<5, Added<3, SizeBytes<1, String>>>
    );

    // unit struct
    #[serializable]
    struct Test3;
}

#[test]
fn versioning() {
    // The example struct is serialized into a Vec<u8> with version 0
    let bytes0 = {
        let revision = 0;
        #[serializable]
        #[derive(PartialEq, Debug)]
        struct Example {
            a: f32,
            b: String,
            c: u32
        }
        let mut s = Serializer::new(Vec::new(), revision).unwrap();
        Example {
            a: 5.4,
            b: "This is a string".to_owned(),
            c: 12345
        }.serialize(&mut s).unwrap();
        s.finish()
    };

    // In revision 1, field b was removed and field d was added.
    // The example struct is still successfully deserialized from the data created in revision 0.
    // After that a new instance is created and serialized
    let bytes1 = {
        let revision = 1;
        #[serializable]
        #[derive(PartialEq, Debug)]
        struct Example {
            a: f32,
            b: Removed<1, String>,
            c: u32,
            d: Added<1, u8>
        }
        let mut d = Deserializer::new(bytes0.as_slice()).unwrap();
        let from0: Example = d.deserialize().unwrap();
        assert_eq!(from0, Example {
            a: 5.4,
            c: 12345,
            d: 0
        });
        assert_eq!(d.finish().len(), 0);

        let mut s = Serializer::new(Vec::new(), revision).unwrap();
        Example {
            a: 12.34,
            c: 56,
            d: 78
        }.serialize(&mut s).unwrap();
        s.finish()
    };

    // In revision 2 'b' was re-added and 'c' was removed again.
    // The data from revision 0 as well as revision 1 is still successfully deserialized 
    {
        let _revision = 2;
        #[serializable]
        #[derive(PartialEq, Debug)]
        struct Example {
            a: f32,
            b: Added<2, Removed<1, String>>,
            c: u32,
            d: Removed<2, Added<1, u8>>
        }

        let mut d0 = Deserializer::new(bytes0.as_slice()).unwrap();
        let example0: Example = d0.deserialize().unwrap();
        assert_eq!(example0, Example {
            a: 5.4,
            b: "This is a string".to_owned(),
            c: 12345
        });
        assert_eq!(d0.finish().len(), 0);

        let mut d1 = Deserializer::new(bytes1.as_slice()).unwrap();
        let example1: Example = d1.deserialize().unwrap();
        assert_eq!(example1, Example {
            a: 12.34,
            b: "".to_owned(),
            c: 56
        });
        assert_eq!(d1.finish().len(), 0);
    };
}


#[test]
fn size_bytes() {
    let str_a = "Hello";
    let str_b = "Goodbye";
    let str_c = "binverse";
    let str_d = "äöü, 😃, 你好，世界";
    
    // revision 0
    let data0 = {
        #[serializable]
        struct Example {
            a: SizeBytes<1, String>,
            b: SizeBytes<2, String>,
            c: SizeBytes<4, String>,
            d: SizeBytes<8, String>
        }

        let mut s0 = Serializer::new(Vec::new(), 0).unwrap();
        Example {
            a: str_a.to_owned(),
            b: str_b.to_owned(),
            c: str_c.to_owned(),
            d: str_d.to_owned()
        }.serialize(&mut s0).unwrap();
        let data0 = s0.finish();
        
        assert_eq!(data0.len(),
            4 + // revision
            1 + str_a.len() + 
            2 + str_b.len() + 
            4 + str_c.len() +
            8 + str_d.len()
        );
        data0
    };

    // revision 1
    {
        #[serializable]
        #[derive(PartialEq, Debug)]
        struct Example {
            a: Removed<1, SizeBytes<1, String>>,
            b: SizeBytes<2, String>,
            c: Removed<1, SizeBytes<4, String>>,
            d: SizeBytes<8, String>
        }

        let mut d = Deserializer::new(data0.as_slice()).unwrap();
        let example0: Example = d.deserialize().unwrap();
        assert_eq!(example0, Example {
            b: str_b.to_owned(),
            d: str_d.to_owned()
        });
    }
}