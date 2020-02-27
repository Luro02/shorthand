use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(verify(ffn = "Self::verify_field"))]
struct Example {
    field: usize,
}

fn main() {}
