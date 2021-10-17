pub mod streams;
pub mod serialize;
pub mod primitives;



#[cfg(test)]
mod test {
    use std::fmt::Debug;
    use rand::prelude::*;
    use crate::{serialize::{SizeBytes, Serialize, Deserialize, SizedSerialize, SizedDeserialize}, streams::{Deserializer, Serializer}};

    fn reserialize_test<T : Serialize + Deserialize + PartialEq + Debug>(val: T, name: &str) {
        let mut s = Serializer::new(Vec::new(), 0);
        val.serialize(&mut s);
        let buf = s.finish();
        let mut d = Deserializer::new(buf.as_slice());
        let new_val: T = d.deserialize();
        assert_eq!(val, new_val, "{}", name);
    }

    fn reserialize_sized_test<T : SizedSerialize + SizedDeserialize + PartialEq + Debug>(val: T, sb: SizeBytes, name: &str) {
        let mut s = Serializer::new(Vec::new(), 0);
        s.serialize_sized(sb, &val);
        let buf = s.finish();
        let mut d = Deserializer::new(buf.as_slice());
        let new_val: T = d.deserialize_sized(sb);
        assert_eq!(val, new_val, "{}", name);
    }

    fn test_n<T>(n: usize) 
    where rand::distributions::Standard: Distribution<T>, T: Serialize + Deserialize + PartialEq + Debug {
        let mut r = rand::thread_rng();
        for _ in 0..n {
            reserialize_test(r.gen::<T>(), std::any::type_name::<T>());
        }
    }

    #[test]
    fn primitive_serialization() {
        test_n::<  u8>(10000);
        test_n::<  i8>(10000);
        
        test_n::< u16>(10000);
        test_n::< i16>(10000);
        
        test_n::< u32>(10000);
        test_n::< i32>(10000);
        
        test_n::< u64>(10000);
        test_n::< i64>(10000);
        
        test_n::<u128>(10000);
        test_n::<i128>(10000);

        test_n::< f32>(10000);
        test_n::< f64>(10000);

        let string = "A random example string";
        reserialize_sized_test(string.to_owned(), SizeBytes::One, "String");
        reserialize_sized_test(string.to_owned(), SizeBytes::Two, "String");
        reserialize_sized_test(string.to_owned(), SizeBytes::Four, "String");
        reserialize_sized_test(string.to_owned(), SizeBytes::Eight, "String");

        reserialize_sized_test("a".repeat(255), SizeBytes::One, "String");
    }
}