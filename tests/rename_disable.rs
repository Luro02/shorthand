use shorthand::ShortHand;

#[derive(ShortHand, Default)]
#[shorthand(rename(format = "hello_{}_world"))]
struct Command {
    first: usize,
    #[shorthand(disable(rename))]
    second: usize,
}

#[test]
fn test_rename_disable() {
    let _: usize = Command::default().hello_first_world();
    let _: &mut Command = Command::default().set_hello_first_world(0);

    let _: usize = Command::default().second();
    let _: &mut Command = Command::default().set_second(0);
}

// TODO: test/define behaviour when `format`, `get` and `set` overwrite each
// other

fn main() {}
