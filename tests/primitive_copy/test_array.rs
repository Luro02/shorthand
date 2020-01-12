use shorthand::ShortHand;

#[derive(ShortHand)]
struct Example {
    value_1: [u8; 0],
    value_2: [u8; 512],
    value_3: [u8; 1024],
}

impl Example {
    fn default() -> Self {
        Self {
            value_1: [0_u8; 0],
            value_2: [0_u8; 512],
            value_3: [0_u8; 1024],
        }
    }
}

fn main() {
    let _: [u8; 0] = Example::default().value_1();
    let _: [u8; 512] = Example::default().value_2();
    let _: [u8; 1024] = Example::default().value_3();
}
