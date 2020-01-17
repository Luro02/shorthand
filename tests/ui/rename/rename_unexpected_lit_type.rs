use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename(b"binary"))]
    value: String,
}

fn main() {}
