#[test]
fn varints() {
    use std::io::{Cursor, Seek};
    use binverse::varint;

    fn test_varint(x: u64) {
        let mut c = Cursor::new(vec![0; 11]);
        varint::write(x, &mut c).unwrap();
        c.rewind().unwrap();
        let x2 = varint::read(&mut c).unwrap_or_else(|e| panic!("Got error while testing varints: {:?}, with value: {}, bytes: {:?}", e, x, c.clone().into_inner()));
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
        test_varint(x);
    }
}