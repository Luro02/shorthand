#![deny(unused_must_use)]
use shorthand::ShortHand;

#[derive(Default, ShortHand)]
#[shorthand(enable(must_use))]
struct Example {
    value: String,
}

fn test_must_use_fail() {
    // should fail because it is marked as unused
    Example::default().value();
}

fn test_should_be_fine() {
    Example::default().set_value("x".to_string());
}

fn main() {}
