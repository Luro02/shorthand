use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(verify(fn = b"Self::verify_field"))]
struct Example {
    field: usize,
}

fn main() {}
