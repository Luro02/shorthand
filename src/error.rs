use core::fmt;
use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::{Lit, Meta};

use crate::utils::ErrorExt;

/// Creates `"`a`, `b` or `c`"` from `&["a", "b", "c"]` and `"or"`
fn format_items<I, T: fmt::Display, L: fmt::Display>(iterator: &I, last: L) -> String
where
    for<'a> &'a I: IntoIterator<Item = &'a T>,
{
    let mut result = String::new();
    let mut iterator = iterator.into_iter().enumerate().peekable();

    while let Some((i, value)) = iterator.next() {
        if i == 0 {
            result.push_str(&format!("`{}`", value));
        } else if iterator.peek().is_some() {
            result.push_str(&format!(", `{}`", value));
        } else {
            result.push_str(&format!(" {} `{}`", last, value));
        }
    }

    result
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    span: Option<Span>,
}

#[derive(Debug)]
enum ErrorKind {
    Custom(String),
    SynError(syn::Error),
    Multiple(Vec<Error>),
    UnknownField {
        found: String,
        expected: Vec<String>,
    },
    UnexpectedType {
        found: String,
        expected: Option<String>,
    },
    UnexpectedMeta {
        format: &'static str,
        expected: Vec<String>,
    },
    UnexpectedField {
        found: String,
        expected: Vec<String>,
    },
    RedundantField {
        field: Cow<'static, str>,
        state: Option<Cow<'static, str>>,
    },
    DuplicateField {
        field: Cow<'static, str>,
    },
}

impl Error {
    const fn new(kind: ErrorKind) -> Self { Self { kind, span: None } }

    pub fn custom<T: fmt::Display>(message: T) -> Self {
        Self::new(ErrorKind::Custom(message.to_string()))
    }

    pub fn unknown_field(value: &str) -> Self {
        Self::new(ErrorKind::UnknownField {
            found: value.to_string(),
            expected: vec![],
        })
    }

    pub fn unexpected_field(value: &str) -> Self {
        Self::new(ErrorKind::UnexpectedField {
            found: value.to_string(),
            expected: vec![],
        })
    }

    pub fn duplicate_field<T>(value: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self::new(ErrorKind::DuplicateField {
            field: value.into(),
        })
    }

    pub fn redundant_field<K, T>(field: K, state: Option<T>) -> Self
    where
        K: Into<Cow<'static, str>>,
        T: Into<Cow<'static, str>>,
    {
        Self::new(ErrorKind::RedundantField {
            field: field.into(),
            state: state.map(Into::into),
        })
    }

    pub fn unexpected_type<T: ToString>(value: &T) -> Self {
        Self::new(ErrorKind::UnexpectedType {
            found: value.to_string(),
            expected: None,
        })
    }

    pub fn unexpected_meta(value: &Meta) -> Self {
        let format = {
            match &value {
                Meta::Path(_) => "Path",
                Meta::List(_) => "List",
                Meta::NameValue(_) => "NameValue",
            }
        };

        Self::new(ErrorKind::UnexpectedMeta {
            format,
            expected: vec![],
        })
        .with_span(&value)
    }

    pub fn unexpected_lit(lit: &syn::Lit) -> Self {
        let found = {
            match lit {
                Lit::Str(_) => "string",
                Lit::ByteStr(_) => "byte string",
                Lit::Byte(_) => "byte",
                Lit::Char(_) => "char",
                Lit::Int(_) => "int",
                Lit::Float(_) => "float",
                Lit::Bool(_) => "bool",
                Lit::Verbatim(_) => "verbatim",
            }
        };

        Self::unexpected_type(&found).with_span(lit)
    }

    pub fn multiple<T>(errors: T) -> Self
    where
        T: IntoIterator<Item = Self>,
    {
        let errors = errors.into_iter().collect::<Vec<_>>();

        if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            Self::new(ErrorKind::Multiple(errors))
        }
    }
}

impl Error {
    pub fn with_span<T: Spanned>(mut self, node: &T) -> Self {
        if self.span.is_none() {
            self.span = Some(node.span());
        }

        self
    }

    pub fn syn(value: syn::Error) -> Self {
        let span = value.span();
        Self::new(ErrorKind::SynError(value)).with_span(&span)
    }

    pub fn into_token_stream(self) -> TokenStream {
        self.flatten()
            .into_iter()
            .map(|e| e.into_syn_error().to_compile_error())
            .collect()
    }

    pub fn span(&self) -> Span { self.span.unwrap_or_else(Span::call_site) }

    fn into_syn_error(self) -> syn::Error {
        if let ErrorKind::Multiple(errors) = self.kind {
            syn::Error::multiple(errors.into_iter().map(|e| syn::Error::new(e.span(), e)))
        } else {
            syn::Error::new(self.span(), self)
        }
    }

    pub fn with_alts<T, K>(mut self, alts: T) -> Self
    where
        T: IntoIterator<Item = K>,
        K: ToString,
    {
        match &mut self.kind {
            ErrorKind::UnknownField {
                found: _found,
                expected,
            } => {
                *expected = alts.into_iter().map(|alt| alt.to_string()).collect();
            }
            ErrorKind::UnexpectedType {
                found: _found,
                expected,
            } => {
                *expected = alts.into_iter().next().map(|v| v.to_string());
            }
            ErrorKind::UnexpectedMeta {
                format: _format,
                expected,
            } => {
                *expected = alts.into_iter().map(|alt| alt.to_string()).collect();
            }
            _ => {}
        };

        self
    }

    fn into_vec(self) -> Vec<Self> {
        if let ErrorKind::Multiple(errors) = self.kind {
            let mut flattened_errors = Vec::with_capacity(errors.capacity());

            for error in errors {
                flattened_errors.extend(error);
            }

            flattened_errors
        } else {
            vec![self]
        }
    }

    fn flatten(self) -> Self {
        let result = self.into_vec();

        if result.len() == 1 {
            result.into_iter().next().unwrap()
        } else {
            Self::multiple(result)
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Custom(value) => value.fmt(f),
            Self::SynError(value) => value.fmt(f),
            Self::Multiple(values) => {
                if values.len() == 1 {
                    values[0].fmt(f)
                } else if values.is_empty() {
                    unreachable!("ErrorKind::Multiple is empty!");
                } else {
                    write!(f, "multiple errors: {}", format_items(values, "and"))?;

                    Ok(())
                }
            }
            Self::UnknownField { found, .. } => write!(f, "unknown field `{}`", found),
            Self::UnexpectedType { found, .. } => write!(f, "unexpected literal type `{}`", found),
            Self::UnexpectedMeta { format, expected } => {
                write!(f, "unexpected meta-item format `{}`", format)?;

                if !expected.is_empty() {
                    write!(f, ", expected {}", format_items(expected, "or"))?;
                }

                Ok(())
            }
            Self::UnexpectedField { found, .. } => write!(f, "unexpected field `{}`", found),
            Self::RedundantField { field, state } => {
                write!(f, "redundant field `{}`", field)?;

                if let Some(state) = state {
                    write!(f, ", which is already {}", state)?;
                }

                Ok(())
            }
            Self::DuplicateField { field } => write!(f, "duplicate field `{}`", field),
        }
    }
}

impl IntoIterator for Error {
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    type Item = Self;

    fn into_iter(self) -> Self::IntoIter { self.into_vec().into_iter() }
}

impl Into<syn::Error> for Error {
    fn into(self) -> syn::Error { self.into_syn_error() }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self { Self::syn(value) }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool { self.kind == other.kind }
}

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool { self.to_string() == other.to_string() }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.kind) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_format_items() {
        assert_eq!(
            format_items(&["a", "b", "c"], "or"),
            "`a`, `b` or `c`".to_string()
        );
        assert_eq!(format_items(&["a", "b"], "or"), "`a` or `b`".to_string());
        assert_eq!(format_items(&["a"], "or"), "`a`".to_string());
        assert_eq!(format_items::<_, u8, _>(&[], "or"), "".to_string());
    }

    #[test]
    fn test_flatten() {
        assert_eq!(
            Error::unknown_field("hello").flatten(),
            Error::unknown_field("hello")
        );

        assert_eq!(
            Error::multiple(vec![Error::unknown_field("zero"), Error::custom("two")]).flatten(),
            Error::multiple(vec![Error::unknown_field("zero"), Error::custom("two")])
        );

        assert_eq!(
            Error::multiple(vec![
                Error::multiple(vec![
                    Error::unknown_field("mash"),
                    Error::unknown_field("hello"),
                    Error::multiple(vec![
                        Error::unknown_field("mash"),
                        Error::unknown_field("hello"),
                    ]),
                ]),
                Error::unknown_field("zero"),
            ])
            .flatten(),
            Error::multiple(vec![
                Error::unknown_field("mash"),
                Error::unknown_field("hello"),
                Error::unknown_field("mash"),
                Error::unknown_field("hello"),
                Error::unknown_field("zero"),
            ])
        );
    }
}
