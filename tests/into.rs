use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(into))]
struct Example {
    value: String,
    other: Option<String>,
}

#[test]
fn test_into() {
    let mut example = Example {
        value: "ex".to_string(),
        other: None,
    };

    let _: &mut Example = example.set_value("hi");
    let _: &mut Example = example.set_other(Some("hi"));

    assert_eq!(example.value(), &"hi".to_string());
    assert_eq!(example.other(), Some(&"hi".to_string()));
}
