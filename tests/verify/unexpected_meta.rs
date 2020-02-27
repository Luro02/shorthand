use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(verify(fn))]
struct Example {
    field: usize,
}

fn main() {}
