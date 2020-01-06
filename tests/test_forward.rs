use shorthand::ShortHand;

#[shorthand(enable(forward(doc)))]
/// This should be in the output
#[shorthand(disable(forward(doc)))]
/// This line should not.
#[shorthand(enable(forward(doc)))]
/// This should be in the output
#[derive(ShortHand)]
struct Example {
    value: String,
}

fn main() {}
