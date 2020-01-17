use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename(format = b"binary"))]
    first: usize,
}

fn main() {}
