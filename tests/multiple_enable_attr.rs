use shorthand::ShortHand;

#[derive(ShortHand, Default)]
#[shorthand(enable(into))]
#[shorthand(enable(copy))]
pub struct Command {
    index: usize,
    optional: Option<usize>,
}

#[test]
fn test_multiple_enable() {
    let _: Option<usize> = Command::default().optional();
    let _: &mut Command = Command::default().set_index(0_u8);
}

fn main() {}
