use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(garbage))]
#[shorthand(disable(garbage))]
pub struct Command {
    value: String,
}

fn main() {}
