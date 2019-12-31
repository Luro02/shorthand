use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Command {
    value: String,
}

#[test]
fn test_set() {
    let _: &mut Command = Command::default().set_value("".to_string());
}

fn main() {}
