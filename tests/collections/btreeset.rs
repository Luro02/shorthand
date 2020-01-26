use shorthand::ShortHand;
use std::collections::BTreeSet;

#[derive(ShortHand, Default)]
#[shorthand(enable(collection_magic))]
struct Example {
    collection: BTreeSet<&'static str>,
    irrelevant_field: usize,
}

fn main() { let _: &mut Example = Example::default().insert_collection("value"); }
