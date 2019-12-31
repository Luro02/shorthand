use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename(format))]
    #[shorthand(rename(format(xyz)))]
    first: usize,
}

fn main() {}
