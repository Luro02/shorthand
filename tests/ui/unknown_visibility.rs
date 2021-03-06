use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(visibility(garbage))]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    index: usize,
    optional: Option<String>,
    current_dir: String,
}

fn main() {}
