# BinVerSe (**Bin**ary **Ver**sioned **Se**rializer)

Provides fast binary serialization with versioning to store data in a backwards-compatible, compact way.

Right now, the crate is still work in progress and I wouldn't recommend using it for larger projects as breaking changes and problems might occur.

## Features
- [x] Simple, fast binary serialization
- [x] Versioning using revision numbers
- [x] Error handling
- [x] Procedural macros to avoid boilerplate code
- [ ] Versioning/size attributes using macros


## Basic example

```rust
use binverse::{serialize::Serialize, streams::{Deserializer, Serializer}};
use binverse_derive::serializable;

fn main() {
    // Add #[serializable] for automatic Serialize/Deserialize
    // implementations and version handling.
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

    // Create a serializer that writes into a Vec<u8>, could be replaced by
    // a file/network stream etc.
    let mut serializer = Serializer::new(Vec::new(), 0).unwrap();
    
    // Serialize the example struct into the serializer.
    example.serialize(&mut serializer).unwrap();

    // Get back the Vec<u8>.
    let data = serializer.finish();
    
    // The length of the data is pretty predictable:
    assert_eq!(data.len(), 
        4    // bytes for revision so the data can be deserialized in future versions 
        + 4  // a: i32       
        + 4  // b: f32
        + 1  // length of the following string (saved using VarInt, can be changed to a constant byte size)
        // the bytes of the string:
        + "Hello binverse!".len()
    );

    // Create a deserializer to recreate the Example instance using the data.
    let mut deserializer = Deserializer::new(data.as_slice()).unwrap();

    // Deserialize the struct. 
    let example_deserialized: Example = deserializer.deserialize().unwrap();
    
    // Both versions match
    assert_eq!(example, example_deserialized);
}


```