use syn::parse::{Parse, ParseStream};
use syn::{Error, Lit, Meta, NestedMeta, Token};

use crate::parser::parse_shorthand;
use crate::utils::{ErrorExt, PathExt};

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
            0 => {
                Err(Error::new_spanned(
                    lit_str,
                    "Missing `{}` in format string.",
                ))
            }
            1 => Ok(lit_str.value()),
            _ => {
                Err(Error::new_spanned(
                    lit_str,
                    "More than one `{}` in format string.",
                ))
            }
        }
    } else {
        // TODO: maybe parse through all arguments at once and then
        // return an       error?
        Err(Error::new_spanned(
            &lit,
            // TODO:
            darling::Error::unexpected_lit_type(&lit),
        ))
    }
}

// #[shorthand(rename("prefix_{}_suffix"))]
// #[shorthand(rename(format = "prefix_{}_suffix"))]
// rename(get = "prefix_{}_suffix")
// rename(set = "prefix_{}_suffix")
// rename(try_set = "prefix_{}_suffix")
// rename(get_mut = "prefix_{}_suffix")
impl Parse for Rename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self::default();
        let input = parse_shorthand(input)?;

        let mut errors = vec![];

        for meta in input.parse_terminated::<Meta, Token![,]>(Meta::parse)? {
            if !meta.path().is_ident("rename") {
                continue;
            }

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
                                    errors.push(Error::new_spanned(
                                        &pair.path,
                                        format!("Unknown field: `{}`", pair.path.to_string()),
                                    ));
                                }
                            } else {
                                let format = {
                                    match meta {
                                        Meta::Path(_) => "Path",
                                        Meta::List(_) => "List",
                                        _ => unreachable!(),
                                    }
                                };

                                errors.push(Error::new_spanned(
                                    &meta,
                                    format!("Unexpected meta-item format `{}`", format),
                                ));
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
                errors.push(Error::new_spanned(&meta, "Invalid format"));
            }
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(Error::multiple(errors))
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
