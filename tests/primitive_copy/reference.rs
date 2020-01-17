use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example<'a> {
    value_0: &'a str,
    value_1: &'static str,
}

fn main() {
    let _: &'_ str = Example::default().value_0();
    let _: &'static str = Example::default().value_1();
}
