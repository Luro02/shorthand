use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename(garbage = ""))]
    first: usize,
}

fn main() {}
