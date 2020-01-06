//! This module is for the visibility attribute

use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Visibility};

use crate::parser::parse_shorthand;

mod kw {
    syn::custom_keyword!(visibility);
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FieldVisibility {
    pub visibility: Visibility,
}

impl FieldVisibility {
    pub fn into_inner(self) -> Visibility { self.visibility }
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

        Ok(Self {
            visibility: content.parse::<LitStr>()?.parse()?,
        })
    }
}

impl Default for FieldVisibility {
    fn default() -> Self {
        Self {
            visibility: syn::parse_str("pub").unwrap(),
        }
    }
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
            syn::parse_str("pub").unwrap()
        );

        // TODO: add more tests, for example what should fail
    }
}
