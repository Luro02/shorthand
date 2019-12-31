use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(try_into))]
struct DoNotDoThis<VALUE> {
    value: VALUE,
}

fn main() {}
