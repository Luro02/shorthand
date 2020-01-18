use shorthand::ShortHand;

#[derive(ShortHand, Default, PartialEq, Debug)]
#[shorthand(enable(try_into))]
struct Example {
    value: String,
    other: Option<String>,
}

#[test]
fn test_try_into() {
    let _: Result<&mut Example, ::core::convert::Infallible> = Example::default().try_value("s");

    assert_eq!(
        Ok(&mut Example {
            value: "hi".to_string(),
            other: None,
        }),
        Example::default().try_value::<&'static str>("hi")
    );
}
