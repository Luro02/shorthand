use shorthand::ShortHand;
use std::collections::BTreeMap;

#[derive(ShortHand, Default)]
#[shorthand(enable(collection_magic))]
struct Example {
    collection: BTreeMap<&'static str, usize>,
}

fn main() { let _: &mut Example = Example::default().insert_collection("value", 0_usize); }
