use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable("string"))]
pub struct Command {
    value: String,
}

fn main() {}
