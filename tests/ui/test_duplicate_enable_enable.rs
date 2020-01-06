use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(copy))]
struct Example {
    // enabling an attribute twice
    #[shorthand(enable(const_fn, const_fn))]
    value_1: usize,
    #[shorthand(enable(const_fn))]
    #[shorthand(enable(const_fn))]
    value_2: usize,
    // enabling an attribute, that is already enabled globally
    #[shorthand(enable(copy))]
    value_3: usize,
}

fn main() {}
