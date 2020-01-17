use core::fmt;
use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::{Lit, Meta};

use crate::utils::ErrorExt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    span: Option<Span>,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool { self.kind == other.kind }
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
    MissingField(String),
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
}

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool { self.to_string() == other.to_string() }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Custom(value) => value.fmt(f),
            Self::SynError(value) => value.fmt(f),
            Self::Multiple(values) => {
                if values.len() == 1 {
                    values[0].fmt(f)
                } else if let Some(first_value) = values.get(0) {
                    write!(f, "multiple errors: {}", first_value)?;

                    for value in values.iter().skip(1) {
                        write!(f, ", {}", value)?;
                    }

                    Ok(())
                } else {
                    unreachable!("ErrorKind::Multiple is empty!");
                }
            }
            Self::UnknownField { found, .. } => {
                write!(f, "unknown field `{}`", found)
                // write!(
                //     f,
                //     ". Did you mean `{}`",
                //     expected.get(0).unwrap_or(&"".to_string())
                // )
            }
            Self::UnexpectedType { found, .. } => write!(f, "unexpected literal type `{}`", found),
            Self::MissingField(field) => write!(f, "missing field `{}`", field),
            Self::UnexpectedMeta { format, expected } => {
                write!(f, "unexpected meta-item format `{}`", format)?;

                if let Some(first_value) = expected.get(0) {
                    write!(f, ", expected ")?;

                    if expected.len() > 1 {
                        write!(f, "[`{}`", first_value)?;
                        for value in expected.iter().skip(1) {
                            write!(f, ", `{}`", value)?;
                        }
                        write!(f, "]")?;
                    } else {
                        write!(f, "`{}`", first_value)?;
                    }
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
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.kind) }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self { Self::syn(value) }
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

    pub fn duplicate_field(value: &str) -> Self {
        // TODO: temporary solution!
        Self::new(ErrorKind::Custom(format!("duplicate field `{}`", value)))
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

    pub fn missing_field(value: &str) -> Self {
        Self::new(ErrorKind::MissingField(value.to_string()))
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

    pub fn unexpected_lit_type(lit: &syn::Lit) -> Self {
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

impl IntoIterator for Error {
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    type Item = Self;

    fn into_iter(self) -> Self::IntoIter { self.into_vec().into_iter() }
}

impl Into<syn::Error> for Error {
    fn into(self) -> syn::Error { self.into_syn_error() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_flatten() {
        assert_eq!(
            Error::unknown_field("hello").flatten(),
            Error::unknown_field("hello")
        );

        assert_eq!(
            Error::multiple(vec![
                Error::unknown_field("zero"),
                Error::missing_field("two")
            ])
            .flatten(),
            Error::multiple(vec![
                Error::unknown_field("zero"),
                Error::missing_field("two")
            ])
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

    #[test]
    #[ignore]
    fn test_error_kind_alts() {
        assert_eq!(
            Error::unknown_field("hello")
                .with_alts(vec!["hallo", "bonjour", "salut", "konichiwa"])
                .into_token_stream()
                .to_string(),
            "".to_string()
        );
    }
}
