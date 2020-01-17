use shorthand::ShortHand;
use std::collections::BTreeSet;

#[derive(ShortHand, Default)]
#[shorthand(enable(collection_magic))]
struct Example {
    collection: BTreeSet<&'static str>,
}

fn main() { let _: &mut Example = Example::default().insert_collection("value"); }
