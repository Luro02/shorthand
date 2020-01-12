use core::fmt;
use std::collections::HashMap;

use quote::quote;

use syn::parse::{Parse, ParseStream};
use syn::{Meta, NestedMeta, Token};

use crate::parser::{parse_enable_disable, parse_shorthand};
use crate::utils::PathExt;

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
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let input = parse_shorthand(input)?;
        let (state, input) = parse_enable_disable(&input)?;

        for nested in input.parse_terminated::<_, Token![,]>(syn::NestedMeta::parse)? {
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
}

impl Forward {
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
        for (key, value) in &other.fields {
            if let Some(v) = self.fields.get_mut(key) {
                *v = *value;
            } else {
                self.fields.insert(key.clone(), *value);
            }
        }

        self
    }

    pub fn is(&self, input: &str) -> bool {
        let result = self.default_state;

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
        let _forward: Forward = syn::parse_str("#[shorthand(enable(forward))]").unwrap();
        let _forward: Forward = syn::parse_str("#[shorthand(enable(forward(x, y, z)))]").unwrap();
    }
}
