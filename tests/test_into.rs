use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(into))]
struct Example {
    value: String,
}

#[test]
fn test_into() {
    let mut example = Example {
        value: "ex".to_string(),
    };

    example.set_value("hi");

    assert_eq!(example.value(), &"hi".to_string());
}
