use shorthand::ShortHand;
use std::collections::HashMap;

#[derive(ShortHand, Default)]
#[shorthand(enable(collection_magic))]
struct Example {
    collection: HashMap<&'static str, usize>,
    irrelevant_field: usize,
}

fn main() { let _: &mut Example = Example::default().insert_collection("value", 0_usize); }
