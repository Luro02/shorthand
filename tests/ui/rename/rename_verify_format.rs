use shorthand::ShortHand;

#[derive(ShortHand)]
struct Command {
    #[shorthand(rename(format = "abc_{}_{}"))]
    first: usize,
    #[shorthand(rename(format = "def"))]
    second: usize,
    #[shorthand(rename(set = "ghi_{}_{}"))]
    third: usize,
    #[shorthand(rename(set = "jkl"))]
    fourth: usize,
    #[shorthand(rename(get = "mno_{}_{}"))]
    fifth: usize,
    #[shorthand(rename(get = "pqr"))]
    sixth: usize,
    #[shorthand(rename(get = "#pqr_{}", set = "{}#_", try_set = "abc@", get_mut = "trä_pöe"))]
    seventh: usize,
    #[shorthand(rename(get = "async"))]
    eigth: usize,
}

fn main() {}
