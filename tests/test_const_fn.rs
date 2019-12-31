use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(enable(const_fn))]
pub struct Command {
    number: usize,
}

#[test]
fn test_const_getter() {
    const NUMBER: usize = Command { number: 0 }.number();

    assert_eq!(NUMBER, 0);
}

fn main() {}
