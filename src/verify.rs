use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{Lit, Meta, NestedMeta, Path, Token};

use crate::error::Error;
use crate::parser::parse_shorthand;

/// This struct represents the `Verify` attribute, which looks like this:
///
/// ```text
/// #[shorthand(verify(fn = "path::to::a::function"))]
/// ```
///
/// The path can also be
///
/// ```text
/// #[shorthand(verify(fn = "Self::path::to::a::function"))]
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Verify {
    path: Option<Path>,
}

impl Default for Verify {
    fn default() -> Self { Self { path: None } }
}

impl ToTokens for Verify {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(path) = &self.path {
            tokens.append_all(quote! {
                #path (&self);
            });
        }
    }
}

// #[shorthand(.., verify(fn = ""), ..)]
impl Parse for Verify {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input = parse_shorthand(input)?;

        for nested in input.parse_terminated::<_, Token![,]>(NestedMeta::parse)? {
            if let NestedMeta::Meta(meta) = nested {
                if !meta.path().is_ident("verify") {
                    continue;
                }

                if let Meta::List(list) = meta {
                    let nested = list
                        .nested
                        .first()
                        .ok_or_else(|| syn::Error::new_spanned(&list, "expected items in list"))?;

                    match &nested {
                        NestedMeta::Meta(Meta::NameValue(name_value)) => {
                            if !name_value.path.is_ident("fn") {
                                return Err(syn::Error::new_spanned(
                                    &name_value.path,
                                    "expected `fn`",
                                ));
                            }

                            if let Lit::Str(str_path) = &name_value.lit {
                                return Ok(Self {
                                    path: Some(str_path.parse_with(Path::parse_mod_style)?),
                                });
                            } else {
                                return Err(Error::unexpected_lit(&name_value.lit)
                                    .with_alts(&["string"])
                                    .into());
                            }
                        }
                        NestedMeta::Meta(meta) => {
                            return Err(Error::unexpected_meta(meta)
                                .with_alts(&["NameValue"])
                                .into());
                        }
                        NestedMeta::Lit(lit) => {
                            return Err(Error::unexpected_lit(lit).into());
                        }
                    }
                } else {
                    return Err(Error::unexpected_meta(&meta).with_alts(&["List"]).into());
                }
            }
        }

        unreachable!("could not find `verify` in the attribute")
    }
}
