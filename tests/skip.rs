#![allow(dead_code)]
use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Command {
    #[shorthand(enable(skip))]
    value: String,
}

fn main() {}
