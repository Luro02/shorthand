use shorthand::ShortHand;
use std::collections::HashSet;

#[derive(ShortHand, Default)]
#[shorthand(enable(collection_magic))]
struct Example {
    collection: HashSet<&'static str>,
}

fn main() { let _: &mut Example = Example::default().insert_collection("value"); }
