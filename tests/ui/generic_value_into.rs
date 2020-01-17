use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(into))]
struct DoNotDoThis<VALUE> {
    value: VALUE,
}

fn main() {}
