use core::fmt;
use std::collections::HashMap;

use syn::parse::{Parse, ParseStream};
use syn::{Meta, NestedMeta, Token};

use crate::error::Error;
use crate::parser::{parse_enable_disable, parse_shorthand};
use crate::utils::PathExt;

#[derive(Clone, PartialEq, Eq)]
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
            match nested {
                NestedMeta::Meta(meta) => {
                    if !meta.path().is_ident("forward") {
                        continue;
                    }

                    match meta {
                        // enable(forward)  -> everything will be forwarded, by default
                        // disable(forward) -> nothing will be forwarded, by default
                        Meta::Path(_) => {
                            return Ok(Self {
                                default_state: state,
                                fields: HashMap::new(),
                            });
                        }
                        // #[shorthand(enable(forward(x, y, z)))]
                        Meta::List(list) => {
                            return Ok(Self {
                                default_state: false,
                                fields: {
                                    let iterator = list
                                        .nested
                                        .into_iter()
                                        .map(|v| {
                                            match v {
                                                // enable(forward(x))
                                                NestedMeta::Meta(meta) => {
                                                    if let Meta::Path(p) = meta {
                                                        Ok((p, state))
                                                    } else {
                                                        Err(Error::unexpected_meta(&meta)
                                                            .with_alts(&["Path"]))
                                                    }
                                                }
                                                // enable(forward("x"))
                                                NestedMeta::Lit(lit) => {
                                                    Err(Error::unexpected_lit(&lit))
                                                }
                                            }
                                        })
                                        .collect::<Vec<Result<_, _>>>();

                                    if iterator.iter().any(Result::is_err) {
                                        Err::<_, syn::Error>(
                                            Error::multiple(
                                                iterator.into_iter().filter_map(Result::err),
                                            )
                                            .into(),
                                        )
                                    } else {
                                        Ok(iterator.into_iter().filter_map(Result::ok).collect())
                                    }
                                }?,
                            });
                        }
                        // #[shorthand(enable(forward(x = "")))]
                        Meta::NameValue(_) => {
                            return Err(Error::unexpected_meta(&meta)
                                .with_alts(&["Path", "List"])
                                .into());
                        }
                    }
                }
                // #[shorthand(enable(""))]
                NestedMeta::Lit(lit) => {
                    return Err(Error::unexpected_lit(&lit).into());
                }
            }
        }

        // this is unreachable, because it is checked with Forward::is_forward, that the
        // input is a valid `Forward` attribute, before it is passed to this
        // function via `syn::parse`
        unreachable!("failed to parse `TokenStream`")
    }
}

impl Forward {
    // parses something like this:
    //
    // #[shorthand(enable(forward, x))]
    // #[shorthand(disable(forward, x))]
    pub fn is_forward(meta: &Meta) -> bool {
        if let Meta::List(list) = meta {
            for part in &list.nested {
                if let NestedMeta::Meta(meta) = part {
                    if let Meta::List(list) = meta {
                        for part in &list.nested {
                            if let NestedMeta::Meta(meta) = part {
                                return meta.path().is_ident("forward");
                            }
                        }
                    }
                }
            }
        }

        false
    }

    pub fn update(&mut self, other: Self) -> &mut Self {
        for (key, value) in other.fields {
            if let Some(v) = self.fields.get_mut(&key) {
                *v = value;
            } else {
                self.fields.insert(key, value);
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

    use crate::utils::AttributeExt;

    #[test]
    fn parse_forward() {
        let _: Forward = syn::parse_str("#[shorthand(enable(forward))]").unwrap();
        let _: Forward = syn::parse_str("#[shorthand(disable(forward))]").unwrap();
        let _: Forward = syn::parse_str("#[shorthand(enable(forward(x, y, z)))]").unwrap();
        let _: Forward = syn::parse_str("#[shorthand(disable(forward(x, y, z)))]").unwrap();
    }

    #[test]
    fn is_forward() {
        let valid_attributes = [
            "#[shorthand(enable(forward))]",
            "#[shorthand(disable(forward))]",
            "#[shorthand(enable(forward(x, y, z)))]",
            "#[shorthand(disable(forward(x, y, z)))]",
        ];

        for valid in &valid_attributes {
            assert!(Forward::is_forward(
                &syn::Attribute::from_str(valid)
                    .unwrap()
                    .parse_meta()
                    .unwrap()
            ));
        }

        let invalid_attributes = [
            "#[shorthand(enable(forward_))]",
            "#[shorthand(disable(_forward))]",
            "#[shorthand(enable(xaforward(x, y, z)))]",
            "#[shorthand(forward(x, y, z))]",
            "#[shorthand(forward_everything)]",
        ];

        for invalid in &invalid_attributes {
            assert!(!Forward::is_forward(
                &syn::Attribute::from_str(invalid)
                    .unwrap()
                    .parse_meta()
                    .unwrap()
            ));
        }
    }
}
