use shorthand::ShortHand;

#[derive(ShortHand, Default)]
pub struct Example {
    optional: Option<String>,
}

#[test]
fn test_option_as_ref() { let _: Option<&String> = Example::default().optional(); }

fn main() {}
