use shorthand::ShortHand;

#[derive(ShortHand)]
struct TupleStruct<'a, T>(u64, usize, &'a T);

fn main() {}
