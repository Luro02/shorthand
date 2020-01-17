use shorthand::ShortHand;

#[derive(ShortHand, Default)]
pub struct Example {
    value_01: bool,
    value_02: char,
    value_03: f32,
    value_04: f64,
    value_05: i8,
    value_06: i16,
    value_07: i32,
    value_08: i64,
    value_09: i128,
    value_10: isize,
    value_11: u8,
    value_12: u16,
    value_13: u32,
    value_14: u64,
    value_15: u128,
    value_16: usize,
}

#[test]
fn test_primitive_copy() {
    let _: bool = Example::default().value_01();
    let _: char = Example::default().value_02();
    let _: f32 = Example::default().value_03();
    let _: f64 = Example::default().value_04();
    let _: i8 = Example::default().value_05();
    let _: i16 = Example::default().value_06();
    let _: i32 = Example::default().value_07();
    let _: i64 = Example::default().value_08();
    let _: i128 = Example::default().value_09();
    let _: isize = Example::default().value_10();
    let _: u8 = Example::default().value_11();
    let _: u16 = Example::default().value_12();
    let _: u32 = Example::default().value_13();
    let _: u64 = Example::default().value_14();
    let _: u128 = Example::default().value_15();
    let _: usize = Example::default().value_16();
}

fn main() {}
