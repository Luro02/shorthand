use shorthand::ShortHand;

#[derive(ShortHand, Default, PartialEq, Debug)]
#[shorthand(enable(try_into, into, strip_option))]
struct Example {
    value: String,
    other: Option<String>,
}

#[test]
fn test_strip_option_try_into() {
    assert_eq!(
        Ok(&mut Example {
            value: "".to_string(),
            other: Some("hi".to_string()),
        }),
        Example::default().try_other::<&'static str>("hi")
    );
}

#[test]
fn test_strip_option_into() {
    assert_eq!(
        &mut Example {
            value: "".to_string(),
            other: Some("hi".to_string()),
        },
        Example::default().set_other::<&'static str>("hi")
    );
}
