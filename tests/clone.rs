use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example {
    #[shorthand(enable(clone))]
    value: String,
}

#[test]
fn test_clone() {
    assert_eq!("".to_string(), Example::default().value());
}

fn main() {}
