use syn::parse::{ParseBuffer, ParseStream};
use syn::Token;

mod kw {
    syn::custom_keyword!(forward);
    syn::custom_keyword!(enable);
    syn::custom_keyword!(disable);
    syn::custom_keyword!(shorthand);
}

/// This function parses the `#[shorthand(inner)]` attribute and returns in this
/// example the `inner` as a `ParseBuffer`.
///
/// # Error
///
/// This function will error, if it encounters an invalid
/// character.
pub fn parse_shorthand(input: ParseStream) -> syn::Result<ParseBuffer> {
    input.parse::<Token![#]>()?;
    let content;
    syn::bracketed!(content in input);
    content.parse::<kw::shorthand>()?;

    let result;
    syn::parenthesized!(result in content);
    Ok(result)
}

// TODO: this should parse x = "", y = z, ...
// fn parse_keyword_list() -> Vec<(Path, Value)> {}

pub fn parse_enable_disable(input: ParseStream) -> syn::Result<(bool, ParseBuffer)> {
    let lookahead = input.lookahead1();
    if lookahead.peek(kw::enable) {
        input.parse::<kw::enable>()?;
        let inner;
        syn::parenthesized!(inner in input);
        Ok((true, inner))
    } else if lookahead.peek(kw::disable) {
        input.parse::<kw::disable>()?;
        let inner;
        syn::parenthesized!(inner in input);
        Ok((false, inner))
    } else {
        // TODO: customize/test?
        Err(lookahead.error())
    }
}
