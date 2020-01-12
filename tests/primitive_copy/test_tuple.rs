use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example {
    value_1: (u64, u8, i32),
    value_2: (bool, char, i32, String),
}

fn main() {
    let _: (u64, u8, i32) = Example::default().value_1();
    let _: &(bool, char, i32, String) = Example::default().value_2();
}
