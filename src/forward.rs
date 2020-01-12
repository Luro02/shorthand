use core::fmt;
use std::collections::HashMap;

use quote::quote;

use syn::parse::{Parse, ParseStream};
use syn::{Meta, NestedMeta, Token};

use crate::parser::parse_shorthand;
use crate::utils::PathExt;

mod kw {
    syn::custom_keyword!(enable);
    syn::custom_keyword!(disable);
}

#[derive(Clone, PartialEq)]
pub struct Forward {
    default_state: bool,
    fields: HashMap<syn::Path, bool>,
}

impl Default for Forward {
    fn default() -> Self {
        Self {
            default_state: false,
            fields: {
                let mut result = HashMap::new();

                // whitelist of built-in attributes, that will always be forwarded:
                // (except, if you disable them)
                result.insert(syn::parse_str("doc").unwrap(), true);
                result.insert(syn::parse_str("cfg").unwrap(), true);
                result.insert(syn::parse_str("cfg_attr").unwrap(), true);
                result.insert(syn::parse_str("allow").unwrap(), true);
                result.insert(syn::parse_str("warn").unwrap(), true);
                result.insert(syn::parse_str("deny").unwrap(), true);
                result.insert(syn::parse_str("forbid").unwrap(), true);
                result.insert(syn::parse_str("deprecated").unwrap(), true);
                result.insert(syn::parse_str("inline").unwrap(), true);
                result.insert(syn::parse_str("cold").unwrap(), true);
                result.insert(syn::parse_str("target_feature").unwrap(), true);

                result
            },
        }
    }
}

impl fmt::Debug for Forward {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // it is very hard to read the debug implementation, if the entire structure
        // of `Path` is shown, so it is converted to a `String`.
        f.debug_struct("Forward")
            .field("default_state", &self.default_state)
            .field(
                "fields",
                &self
                    .fields
                    .iter()
                    .map(|(path, state)| (path.to_string(), state))
                    .collect::<HashMap<_, _>>(),
            )
            .finish()
    }
}

impl Parse for Forward {
    // Parses the following format:
    // enable(forward())
    // disable(forward())
    // It ignores all other attributes!
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // parse outer attribute, if it exists (#[shorthand(inner_attribute)])
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![#]) {
            Self::parse_inner(&parse_shorthand(input)?)
        } else if lookahead.peek(kw::enable) || lookahead.peek(kw::disable) {
            Self::parse_inner(input)
        } else {
            // TODO: customize?
            Err(lookahead.error())
            //unimplemented!("Attribute does neither start with `#` nor with
            // `enable`/`disable`")
        }
    }
}

impl Forward {
    pub fn parse_inner(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let state = {
            if lookahead.peek(kw::enable) {
                input.parse::<kw::enable>()?;
                true
            } else if lookahead.peek(kw::disable) {
                input.parse::<kw::disable>()?;
                false
            } else {
                // TODO: customize/test?
                return Err(lookahead.error());
            }
        };

        let inner;
        syn::parenthesized!(inner in input);

        for nested in inner.parse_terminated::<_, Token![,]>(syn::NestedMeta::parse)? {
            if let NestedMeta::Meta(meta) = nested {
                if meta.path().is_ident("forward") {
                    // enable(forward)  -> everything will be forwarded, by default
                    // disable(forward) -> nothing will be forwarded, by default
                    if let Meta::Path(_) = meta {
                        return Ok(Self {
                            default_state: state,
                            fields: HashMap::new(),
                        });
                    } else if let Meta::List(list) = meta {
                        return Ok(Self {
                            default_state: false,
                            fields: list
                                .nested
                                .into_iter()
                                .filter_map(|v| {
                                    // enable(forward(x))
                                    if let NestedMeta::Meta(meta) = v {
                                        Some((meta.path().clone(), state))
                                    // enable(forward("x"))
                                    } else {
                                        // TODO: I think this should error
                                        //       (Literals are unexpected)
                                        None
                                    }
                                })
                                .collect(),
                        });
                    } else {
                        unimplemented!("TODO: Handle this error case");
                    }
                }
            } else {
                // panic or error, I don't know
            }
        }

        //dbg!(input);

        unimplemented!("Nothing inside the attribute.");
    }

    pub fn is_forward(meta: &Meta) -> bool {
        // #[shorthand(enable(forward, x))]
        // #[shorthand(disable(forward, x))]
        // enable(forward, x)
        // disable(forward, x)

        // this should be good enough (parsing the entire TokenStream is overkill)
        for part in quote!(#meta).to_string().replace(')', "").split('(') {
            for attr in part.split(',') {
                if attr.trim() == "forward" {
                    return true;
                }
            }
        }

        false
    }

    pub fn update(mut self, other: &Self) -> Self {
        // dbg!(&self);
        // dbg!(&other);
        for (key, value) in &other.fields {
            if let Some(v) = self.fields.get_mut(key) {
                *v = *value;
            } else {
                self.fields.insert(key.clone(), *value);
            }
        }
        // dbg!(&self);

        self
    }

    pub fn is(&self, input: &str) -> bool {
        let result = self.default_state;

        //dbg!(&self.fields);

        for (key, value) in &self.fields {
            if key.is_ident(input) {
                return *value;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_forward() {
        // TODO: discard this kind of attributes, only absolute ones are supported!
        // #[shorthand(enable(forward))] instead of just enable(forward)
        let _forward: Forward = syn::parse_str("enable(forward)").unwrap();
        let _forward: Forward = syn::parse_str("enable(forward(x, y, z))").unwrap();
    }
}
