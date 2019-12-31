use shorthand::ShortHand;

#[derive(ShortHand)]
enum ExampleEnum {
    First,
    Second,
    Third { a: String, b: usize },
}

fn main() {}
