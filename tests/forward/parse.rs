use shorthand::ShortHand;

#[derive(ShortHand)]
struct Example {
    #[shorthand(enable(forward))] // valid
    value_0: String,
    #[shorthand(enable(forward(doc)))] // valid
    value_1: String,
    #[shorthand(disable(forward(doc)))] // valid
    value_2: String,
    #[shorthand(enable(forward("")))] // invalid
    value_3: String,
    #[shorthand(enable(forward = ""))] // invalid
    value_4: String,
    #[shorthand(enable(forward(x = "")))] // invalid
    value_5: String,
    #[shorthand(enable(""))] // invalid
    value_6: String,
    #[shorthand(enable(forward(x(y))))] // invalid
    value_7: String,
}

fn main() {}
