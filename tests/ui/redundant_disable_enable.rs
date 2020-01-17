use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(option_as_ref, const_fn))]
#[shorthand(disable(option_as_ref, const_fn))]
pub struct Command {
    value: String,
}

fn main() {}
