use shorthand::ShortHand;

#[derive(ShortHand)]
#[shorthand(verify(fn = "Self::verify_field"), enable(try_into))]
struct Example {
    field: usize,
}

impl Example {
    fn verify_field(&self) {
        if self.field == 0 {
            panic!("the field should not be zero");
        }
    }
}

#[test]
#[should_panic = "the field should not be zero"]
fn test_verify_failed_set() {
    let mut example = Example { field: 0 };

    let _: &mut Example = example.set_field(0);
}

#[test]
#[should_panic = "the field should not be zero"]
fn test_verify_failed_try() {
    let mut example = Example { field: 0 };

    let _: &mut Example = example.try_field(0_u8).unwrap();
}

fn main() {}
