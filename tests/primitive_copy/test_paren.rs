#![allow(unused_parens)]
use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example {
    value: (u8),
}

fn main() { let _: (u8) = Example::default().value(); }
