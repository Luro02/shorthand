use shorthand::ShortHand;

#[derive(ShortHand)]
pub struct Command {
    #[shorthand(enable(copy))]
    value: String,
}

fn main() {}
