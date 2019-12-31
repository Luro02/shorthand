use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example {
    #[shorthand(enable(collection_magic))]
    value: Vec<String>,
}

#[test]
fn test_collection_magic_vec() {
    let _: &mut Example = Example::default().push_value("Hi".to_string());
}

fn main() {}
