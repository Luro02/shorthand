use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(rename))]
struct Command {
    value: String,
}

fn main() {}
