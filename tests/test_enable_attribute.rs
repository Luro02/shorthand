use shorthand::ShortHand;

#[derive(ShortHand, Default)]
#[shorthand(enable(option_as_ref))]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    index: usize,
    optional: Option<String>,
    current_dir: String,
}

#[test]
fn test_option_as_ref() {
    let _: Option<&String> = Command::default().optional();
}

fn main() {}
