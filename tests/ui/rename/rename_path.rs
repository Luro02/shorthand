use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename)]
    first: usize,
}

fn main() {}
