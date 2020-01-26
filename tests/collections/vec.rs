use shorthand::ShortHand;
use std::vec::Vec;

#[derive(ShortHand, Default)]
#[shorthand(enable(collection_magic))]
struct Example {
    vec: Vec<&'static str>,
    irrelevant_field: usize,
}

fn main() { let _: &mut Example = Example::default().push_vec("value"); }
