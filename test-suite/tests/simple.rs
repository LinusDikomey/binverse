use binverse::error::BinverseError;

#[test]
fn simple_data() -> Result<(), BinverseError> {
    let bytes = binverse::write_no_revision(Vec::new(), 3_i32)?;
    assert_eq!(binverse::read_no_revision::<_, i32>(bytes.as_slice(), 0)?.0, 3_i32);
    Ok(())
}

#[test]
fn leftover_bytes() {
    let (x, r) = binverse::read_no_revision::<&[u8], i16>(&[1, 2, 3, 4], 0).unwrap();
    assert_eq!(x, 1 | 2 << 8);
    assert_eq!(r, &[3, 4]);
}