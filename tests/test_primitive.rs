#[cfg(test)]
use pretty_assertions::assert_eq;
use shorthand::ShortHand;

#[derive(ShortHand, Default)]
pub struct Command {
    index01: bool,
    index02: char,
    index03: f32,
    index04: f64,
    index05: i8,
    index06: i16,
    index07: i32,
    index08: i64,
    index09: i128,
    index10: isize,
    index11: u8,
    index12: u16,
    index13: u32,
    index14: u64,
    index15: u128,
    index16: usize,
}

#[test]
fn test_bool() {
    assert_eq!(Command::default().index01(), false);
}

#[test]
fn test_char() {
    assert_eq!(Command::default().index02(), '\x00');
}

#[test]
fn test_f32() {
    assert_eq!(Command::default().index03(), 0.0_f32);
}

#[test]
fn test_f64() {
    assert_eq!(Command::default().index04(), 0.0_f64);
}

#[test]
fn test_i8() {
    assert_eq!(Command::default().index05(), 0_i8);
}

#[test]
fn test_i16() {
    assert_eq!(Command::default().index06(), 0_i16);
}

#[test]
fn test_i32() {
    assert_eq!(Command::default().index07(), 0_i32);
}

#[test]
fn test_i64() {
    assert_eq!(Command::default().index08(), 0_i64);
}

#[test]
fn test_i128() {
    assert_eq!(Command::default().index09(), 0_i128);
}

#[test]
fn test_isize() {
    assert_eq!(Command::default().index10(), 0_isize);
}

#[test]
fn test_u8() {
    assert_eq!(Command::default().index11(), 0_u8);
}

#[test]
fn test_u16() {
    assert_eq!(Command::default().index12(), 0_u16);
}

#[test]
fn test_u32() {
    assert_eq!(Command::default().index13(), 0_u32);
}

#[test]
fn test_u64() {
    assert_eq!(Command::default().index14(), 0_u64);
}

#[test]
fn test_u128() {
    assert_eq!(Command::default().index15(), 0_u128);
}

#[test]
fn test_usize() {
    assert_eq!(Command::default().index16(), 0_usize);
}

fn main() {}
