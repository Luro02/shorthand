// TODO: I think I broke cargo expand with this file
//       `cargo expand --test test_rename`
// I think this should be reported as soon as this library releases
use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Command {
    #[shorthand(rename(format = "hello_{}_world"), enable(try_into, get_mut))]
    first: usize,
    #[shorthand(rename(get = "get_{}_suffix"))]
    second: usize,
    #[shorthand(rename(set = "set_{}_suffix"))]
    third: usize,
    #[shorthand(rename(try_set = "try_set_{}_suffix"), enable(try_into))]
    forth: usize,
    #[shorthand(rename(get_mut = "get_{}_mut"), enable(get_mut))]
    fifth: usize,
    #[shorthand(
        rename(
            get = "hello",
            set = "setto",
            try_set = "trysetto",
            get_mut = "getmutte"
        ),
        enable(try_into, get_mut)
    )]
    sixth: usize,
}

#[test]
fn test_no_insert() {
    let _: usize = Command::default().hello();
    let _: &mut Command = Command::default().setto(1_usize);
    let _: Result<&mut Command, core::num::TryFromIntError> = Command::default().trysetto(0_u32);
    let _: &mut usize = Command::default().getmutte();
}

#[test]
fn test_rename_format() {
    let _: usize = Command::default().hello_first_world();
    let _: &mut Command = Command::default().set_hello_first_world(0);
    let _: Result<&mut Command, core::num::TryFromIntError> =
        Command::default().try_hello_first_world(0_u32);
    let _: &mut usize = Command::default().hello_first_world_mut();
}

#[test]
fn test_rename_get() {
    let _: usize = Command::default().get_second_suffix();
    let _: &mut Command = Command::default().set_second(0);
}

#[test]
fn test_rename_set() {
    let _: usize = Command::default().third();
    let _: &mut Command = Command::default().set_third_suffix(0);
}

#[test]
fn test_rename_try_set() {
    let _: usize = Command::default().forth();
    let _: &mut Command = Command::default().set_forth(0);
    let _: Result<&mut Command, core::num::TryFromIntError> =
        Command::default().try_set_forth_suffix::<u32>(0_u32);
}

#[test]
fn test_rename_get_mut() {
    let _: usize = Command::default().fifth();
    let _: &mut Command = Command::default().set_fifth(0);
    let _: &mut usize = Command::default().get_fifth_mut();
}

fn main() {}
