use shorthand::ShortHand;

/// struct level attributes/documentation should not be forwarded
#[derive(ShortHand, Default)]
struct Example {
    /// Data has some special restrictions.
    /// - the String can only exist of uppercase characters
    /// - Numbers are allowed
    /// - The String has a maximum size of 5.
    #[shorthand(disable(forward(doc)))]
    /// This part will not be forwarded.
    #[shorthand(enable(forward(doc)))]
    ///
    /// # Example
    ///
    /// ```
    /// println!("nice");
    /// ```
    data: String,
}

fn main() {}
