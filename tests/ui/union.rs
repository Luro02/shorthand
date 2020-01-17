use shorthand::ShortHand;

#[derive(ShortHand)]
#[repr(C)]
union ExampleUnion {
    f1: u32,
    f2: f32,
}

fn main() {}
