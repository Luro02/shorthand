use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand]
struct Example {
    value: usize,
}

#[derive(ShortHand)]
#[shorthand = ""]
struct Example2 {
    value: usize,
}

fn main() {}
