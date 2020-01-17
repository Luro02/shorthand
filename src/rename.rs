use syn::parse::{Parse, ParseStream};
use syn::{Lit, Meta, NestedMeta, Token};

use crate::error::Error;
use crate::parser::parse_shorthand;
use crate::utils::PathExt;

mod kw {
    syn::custom_keyword!(rename);
}

#[derive(Clone, Debug)]
pub struct Rename {
    pub get_format: String,
    pub set_format: String,
    pub get_mut_format: String,
    pub try_set_format: String,
}

fn parse_format(lit: &Lit) -> syn::Result<String> {
    if let Lit::Str(lit_str) = lit {
        // check for invariants (too many `{}` or missing `{}`)
        match lit_str.value().matches("{}").count() {
            0 => Err(syn::Error::new_spanned(lit_str, "missing `{}`")),
            1 => Ok(lit_str.value()),
            _ => Err(syn::Error::new_spanned(lit_str, "more than one `{}`")),
        }
    } else {
        Err(Error::unexpected_lit(lit).with_span(&lit).into())
    }
}

impl Parse for Rename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self::default();
        let input = parse_shorthand(input)?;

        let mut errors = vec![];

        for meta in input.parse_terminated::<Meta, Token![,]>(Meta::parse)? {
            // ignore other attributes like `#[shorthand(enable)]`
            if !meta.path().is_ident("rename") {
                continue;
            }

            // #[shorthand(rename(x, y, z))]
            if let Meta::List(list) = meta {
                for nested in list.nested {
                    match nested {
                        NestedMeta::Meta(meta) => {
                            if let Meta::NameValue(pair) = meta {
                                if pair.path.is_ident("format") {
                                    let value = parse_format(&pair.lit)?;

                                    result.get_format = value.clone();
                                    result.set_format = format!("set_{}", value);
                                    result.get_mut_format = format!("{}_mut", value);
                                    result.try_set_format = format!("try_{}", value);
                                } else if pair.path.is_ident("get") {
                                    let value = parse_format(&pair.lit)?;

                                    result.get_format = value.clone();
                                } else if pair.path.is_ident("set") {
                                    let value = parse_format(&pair.lit)?;

                                    result.set_format = value.clone();
                                } else if pair.path.is_ident("try_set") {
                                    let value = parse_format(&pair.lit)?;

                                    result.try_set_format = value.clone();
                                } else if pair.path.is_ident("get_mut") {
                                    let value = parse_format(&pair.lit)?;

                                    result.get_mut_format = value.clone();
                                } else {
                                    errors.push(
                                        Error::unknown_field(&pair.path.to_string())
                                            .with_span(&pair.path),
                                    );
                                }
                            } else {
                                errors
                                    .push(Error::unexpected_meta(&meta).with_alts(&["NameValue"]));
                            }
                        }
                        NestedMeta::Lit(lit) => {
                            let value = parse_format(&lit)?;
                            result.get_format = value.clone();
                            result.set_format = format!("set_{}", value);
                            result.get_mut_format = format!("{}_mut", value);
                            result.try_set_format = format!("try_{}", value);
                        }
                    }
                }
            } else {
                // #[shorthand(rename)]
                // #[shorthand(rename = "")]
                errors.push(Error::unexpected_meta(&meta).with_alts(&["List"]));
            }
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(Error::multiple(errors).into())
        }
    }
}

impl Default for Rename {
    fn default() -> Self {
        Self {
            get_format: "{}".to_string(),
            set_format: "set_{}".to_string(),
            get_mut_format: "{}_mut".to_string(),
            try_set_format: "try_{}".to_string(),
        }
    }
}
