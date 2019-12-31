use shorthand::ShortHand;

#[derive(Copy, Clone, Default)]
struct Number(usize);

#[derive(ShortHand, Default)]
struct Command {
    #[shorthand(enable(copy))]
    index: Number,
    #[shorthand(disable(copy))]
    index2: Number,
    index3: Number,
}

#[test]
fn test_copy() {
    let _: Number = Command::default().index();
    let _: &Number = Command::default().index2();
    let _: &Number = Command::default().index3();
}

fn main() {}
