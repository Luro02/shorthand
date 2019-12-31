use core::marker::PhantomData;
use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Command<'a, T: Copy> {
    value: String,
    _p: PhantomData<&'a T>,
}

#[test]
fn test_set() {
    let _: &mut Command<usize> = Command::default().set_value("".to_string());
}

fn main() {}
