//! This module is for the visibility attribute

use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Visibility};

use crate::error::Error;
use crate::parser::parse_shorthand;

mod kw {
    syn::custom_keyword!(visibility);
    syn::custom_keyword!(inherit);
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldVisibility {
    Visible(Visibility),
    Inherit,
}

impl FieldVisibility {
    pub fn into_inner(self) -> Option<Visibility> {
        match self {
            Self::Visible(vis) => Some(vis),
            Self::Inherit => None,
        }
    }
}

impl Parse for FieldVisibility {
    /// Parses an attribute, that looks like this:
    ///
    /// ```text
    /// #[shorthand(visibility("pub"))]
    /// #[shorthand(visibility("pub(crate)"))]
    /// ```
    ///
    /// # Note
    ///
    /// The argument, that is passed to this attribute has to be a
    /// [`LitStr`], because otherwise the following would not work:
    ///
    /// ```text
    /// #[shorthand(visibility(pub(in crate)))]
    /// ```
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sub = parse_shorthand(input)?;
        sub.parse::<kw::visibility>()?;
        let content;
        syn::parenthesized!(content in sub);

        let lookahead = content.lookahead1();

        if let Ok(lit_str) = content.parse::<LitStr>() {
            Ok(Self::Visible(lit_str.parse()?))
        } else if lookahead.peek(kw::inherit) {
            content.parse::<kw::inherit>()?;
            Ok(Self::Inherit)
        } else {
            Err(Error::custom("expected literal or `inherit`")
                .with_span(&lookahead.error().span())
                .into())
        }
    }
}

impl Default for FieldVisibility {
    fn default() -> Self { Self::Visible(syn::parse_str("pub").unwrap()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;

    #[test]
    fn test_parse() {
        assert_eq!(
            syn::parse2::<FieldVisibility>(quote!(#[shorthand(visibility("pub"))]))
                .unwrap()
                .into_inner(),
            Some(syn::parse_str("pub").unwrap())
        );

        // TODO: add more tests, for example what should fail
    }
}
