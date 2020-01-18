use shorthand::ShortHand;

struct NotClone;

#[derive(ShortHand)]
struct Example {
    #[shorthand(enable(clone))]
    value: NotClone,
}

fn main() {}
