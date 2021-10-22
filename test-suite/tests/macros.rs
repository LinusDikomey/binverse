use std::fmt::Debug;

use binverse::{Added, Removed, serialize::Serialize, streams::{Deserializer, Serializer}};
use binverse_derive::serializable;

#[test]
fn basic_struct() {
    #[serializable]
    #[derive(Debug, PartialEq)]
    struct Example {
        a: i32,
        b: Removed<f32, 4>,
        c: String
    }

    let example = Example {
        a: -1253891,
        //b: 44223.125,
        c: String::from("Hello binverse!")
    };

    let mut serializer = Serializer::new(Vec::new(), 0).unwrap();
    example.serialize(&mut serializer).unwrap();
    let data = serializer.finish();

    assert_eq!(data.len(), 
        4    // bytes for revision so the data can be serialized in future versions 
        + 4  // a: i32       
        + 4  // b: f32
        + 1  // length of the following string (saved using VarInt, can be changed to a constant byte size)
        // the bytes of the string:
        + "Hello binverse!".len()
    );

    let mut deserializer = Deserializer::new(data.as_slice()).unwrap();
    let example_deserialized: Example = deserializer.deserialize().unwrap();
    assert_eq!(example, example_deserialized);
    
    assert_eq!(deserializer.finish().len(), 0, "Remaining bytes after deserializing");
}