use shorthand::ShortHand;

#[derive(ShortHand, Default)]
#[shorthand(enable(option_as_ref))]
#[shorthand(enable(copy))]
pub struct Command {
    index: usize,
    optional: Option<usize>,
}

#[test]
fn test_multiple_enable() {
    let _: Option<usize> = Command::default().optional();
    let _: usize = Command::default().index();
}

fn main() {}
