use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example {
    #[shorthand(enable(get_mut))]
    value: String,
}

#[test]
fn test_get_mut() {
    let _: &String = Example::default().value_mut();
}

fn main() {}
