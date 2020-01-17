use shorthand::ShortHand;

#[derive(ShortHand)]
pub struct Command {
    #[shorthand(visibility("pub(crate)"))]
    executable: String,
    #[shorthand(visibility("pub(self)"))]
    args: Vec<String>,
    #[shorthand(visibility("pub(in crate)"))]
    env: Vec<String>,
    index: usize,
    optional: Option<String>,
    current_dir: String,
}

fn main() {}
