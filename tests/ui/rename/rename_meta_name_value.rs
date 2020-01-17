use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename = "example")]
    first: usize,
}

fn main() {}
