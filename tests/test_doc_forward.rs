use shorthand::ShortHand;

#[derive(ShortHand, Default)]
struct Example {
    /// Data has some special restrictions.
    /// - the String can only exist of uppercase characters
    /// - Numbers are allowed
    /// - The String has a maximum size of 5.
    #[shorthand(disable(forward_attributes))]
    /// This part will not be forwarded.
    #[shorthand(enable(forward_attributes))]
    ///
    /// # Example
    ///
    /// ```
    /// println!("nice");
    /// ```
    data: String,
}

fn main() {}
