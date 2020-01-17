// TODO!
use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(const_fn))]
pub struct Command {
    value: String,
}

fn main() {}
