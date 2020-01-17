use shorthand::ShortHand;

#[derive(ShortHand, Default, PartialEq, Debug)]
#[shorthand(enable(try_into))]
struct Example {
    value: String,
}

#[test]
fn test_try_into() {
    assert_eq!(
        Ok(&mut Example {
            value: "hi".to_string()
        }),
        Example::default().try_value::<&'static str>("hi")
    );
}
