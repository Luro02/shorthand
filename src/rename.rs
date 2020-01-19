use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Ident, Lit, Meta, NestedMeta, Token};

use crate::error::Error;
use crate::parser::parse_shorthand;
use crate::utils::{PathExt, Spanned};

mod kw {
    syn::custom_keyword!(rename);
}

#[derive(Clone, Debug)]
pub struct Rename {
    get_format: Format,
    set_format: Format,
    get_mut_format: Format,
    try_set_format: Format,
}

/// Copied from the `syn::Ident` implementation.
fn is_reserved_ident<T: AsRef<str>>(value: T) -> bool {
    match value.as_ref() {
        "_" |
        // Based on https://doc.rust-lang.org/grammar.html#keywords
        // and https://github.com/rust-lang/rfcs/blob/master/text/2421-unreservations-2018.md
        // and https://github.com/rust-lang/rfcs/blob/master/text/2420-unreserve-proc.md
        "abstract" | "as" | "become" | "box" | "break" | "const" | "continue" |
        "crate" | "do" | "else" | "enum" | "extern" | "false" | "final" | "fn" |
        "for" | "if" | "impl" | "in" | "let" | "loop" | "macro" | "match" |
        "mod" | "move" | "mut" | "override" | "priv" | "pub" | "ref" |
        "return" | "Self" | "self" | "static" | "struct" | "super" | "trait" |
        "true" | "type" | "typeof" | "unsafe" | "unsized" | "use" | "virtual" |
        "where" | "while" | "yield" => true,
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub struct Format(Spanned<String>);

impl Format {
    pub fn new<I: IntoIterator<Item = char>>(input: I, span: Span) -> Result<Self, Error> {
        let mut errors = vec![];

        let into_iter = input.into_iter();
        let mut result = String::with_capacity(into_iter.size_hint().0);

        let mut iterator = into_iter.enumerate().peekable();
        let mut last_char = None;

        while let Some((i, c)) = iterator.next() {
            let valid = {
                // only `{` followed by an `}` is valid, a single `{` or `}` is invalid!
                // and those characters are only valid because, `{}` will be replaced by the
                // field name.
                if c == '{' && iterator.peek().map_or(false, |s| s.1 == '}')
                    || c == '}' && last_char.map_or(false, |c| c == '{')
                {
                    true
                } else if i == 0 {
                    if c == '_' {
                        iterator.peek().is_some()
                    } else {
                        // first char must be in the range `a-z` or `A-Z`
                        c.is_ascii_alphabetic()
                    }
                } else {
                    c.is_ascii_alphanumeric() || c == '_'
                }
            };

            if !valid {
                errors.push(
                    Error::custom(format!(
                        "invalid character in format string `{}` at position {}",
                        c,
                        i + 1
                    ))
                    .with_span(&span),
                );
            }

            last_char = Some(c);
            result.push(c);
        }

        if is_reserved_ident(&result) {
            errors.push(Error::custom("this ident is reserved").with_span(&span));
        }

        if errors.is_empty() {
            Ok(Self(Spanned::new(result).with_span(&span)))
        } else {
            Err(Error::multiple(errors))
        }
    }

    pub fn from_lit(lit: &Lit) -> Result<Self, Error> {
        if let Lit::Str(lit_str) = lit {
            Self::new(lit_str.value().chars(), lit_str.span())
        } else {
            Err(Error::unexpected_lit(lit).with_span(&lit))
        }
    }

    pub fn verify_strict(&self) -> Result<(), Error> {
        if self.0.matches("{}").count() >= 1 {
            Ok(())
        } else {
            Err(Error::custom("missing `{}`").with_span(&self.0))
        }
    }

    // TODO: this function allows to create invalid Format!
    pub fn map<F: FnOnce(String) -> String>(self, f: F) -> Self { Self(self.0.map(|s| f(s))) }

    pub fn with_ident(&self, replace: &Ident) -> Ident {
        Ident::new(
            &self
                .0
                .split('{')
                .map(|t| t.replace("}", &replace.to_string()))
                .collect::<String>(),
            self.0.span(),
        )
    }
}

impl Rename {
    // TODO: remove unnecessary Result
    pub fn format_get(&self, value: &Ident) -> Result<Ident, Error> {
        Ok(self.get_format.with_ident(value))
    }

    pub fn format_set(&self, value: &Ident) -> Result<Ident, Error> {
        Ok(self.set_format.with_ident(value))
    }

    pub fn format_get_mut(&self, value: &Ident) -> Result<Ident, Error> {
        Ok(self.get_mut_format.with_ident(value))
    }

    pub fn format_try_set(&self, value: &Ident) -> Result<Ident, Error> {
        Ok(self.try_set_format.with_ident(value))
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
                                let format = {
                                    match Format::from_lit(&pair.lit) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            errors.push(e);
                                            continue;
                                        }
                                    }
                                };

                                if pair.path.is_ident("format") {
                                    if let Err(e) = format.verify_strict() {
                                        errors.push(e);
                                        continue;
                                    }

                                    result.get_format = format.clone();
                                    result.set_format =
                                        format.clone().map(|s| format!("set_{}", s));
                                    result.try_set_format =
                                        format.clone().map(|s| format!("try_{}", s));
                                    result.get_mut_format =
                                        format.clone().map(|s| format!("{}_mut", s));
                                } else if pair.path.is_ident("get") {
                                    result.get_format = format.clone();
                                } else if pair.path.is_ident("set") {
                                    result.set_format = format.clone();
                                } else if pair.path.is_ident("try_set") {
                                    result.try_set_format = format.clone();
                                } else if pair.path.is_ident("get_mut") {
                                    result.get_mut_format = format.clone();
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
                        // #[shorthand(rename("field"))]
                        NestedMeta::Lit(lit) => {
                            let format = {
                                match Format::from_lit(&lit) {
                                    Ok(value) => {
                                        if let Err(e) = value.verify_strict() {
                                            errors.push(e);
                                            continue;
                                        }

                                        value
                                    }
                                    Err(e) => {
                                        errors.push(e);
                                        continue;
                                    }
                                }
                            };

                            result.get_format = format.clone();
                            result.set_format = format.clone().map(|s| format!("set_{}", s));
                            result.try_set_format = format.clone().map(|s| format!("try_{}", s));
                            result.get_mut_format = format.clone().map(|s| format!("{}_mut", s));
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
            get_format: Format::new("{}".chars(), Span::call_site()).unwrap(),
            set_format: Format::new("set_{}".chars(), Span::call_site()).unwrap(),
            get_mut_format: Format::new("{}_mut".chars(), Span::call_site()).unwrap(),
            try_set_format: Format::new("try_{}".chars(), Span::call_site()).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::ErrorExt;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_rename() {
        let valid_attributes = &[
            r#"#[shorthand(rename(format = "abc_{}_cde"))]"#,
            r#"#[shorthand(rename(get = "abc_{}"))]"#,
            r#"#[shorthand(rename(set = "set_{}"))]"#,
            r#"#[shorthand(rename(get = "xyz_{}", set = "set_{}"))]"#,
        ];

        for attr in valid_attributes {
            syn::parse_str::<Rename>(attr).unwrap();
        }

        assert_eq!(
            Error::syn(
                syn::parse_str::<Rename>("#[shorthand(rename(get = \"#this_is#in@lid\"))]")
                    .expect_err("invalid rename attribute could be parsed!")
            ),
            Error::syn(syn::Error::multiple(vec![
                syn::Error::new(
                    Span::call_site(),
                    "invalid character in format string `#` at position 1"
                ),
                syn::Error::new(
                    Span::call_site(),
                    "invalid character in format string `#` at position 9"
                ),
                syn::Error::new(
                    Span::call_site(),
                    "invalid character in format string `@` at position 12"
                ),
            ]))
        );
    }
}
