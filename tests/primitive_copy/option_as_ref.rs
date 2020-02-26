use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example<'a> {
    value: Option<&'a str>,
    other: Option<usize>,
}

#[test]
fn test_option_copy_as_ref() {
    let _: Option<&str> = Example::default().value();
    let _: Option<usize> = Example::default().other();
}

fn main() {}
