use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::Parser;
use syn::{Attribute, Ident, Meta, Path, PathArguments, Type};

use core::ops::{Deref, DerefMut};

/// Attach a [`Span`] to an arbitrary type.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Spanned<T> {
    inner: T,
    span: Option<Span>,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

/// Allows to compare for example a `Spanned<bool>` with a `bool`
impl<T: PartialEq> PartialEq<T> for Spanned<T> {
    fn eq(&self, other: &T) -> bool { &self.inner == other }
}

impl<T: Eq> Eq for Spanned<T> {}

impl<T: ::core::hash::Hash> ::core::hash::Hash for Spanned<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: ::core::hash::Hasher,
    {
        self.inner.hash(state)
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl<T> syn::spanned::Spanned for Spanned<T> {
    fn span(&self) -> Span { self.span.unwrap_or_else(Span::call_site) }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl<T> Spanned<T> {
    pub const fn new(inner: T) -> Self { Self { inner, span: None } }

    pub fn with_span<S: ::syn::spanned::Spanned>(mut self, span: &S) -> Self {
        if self.span.is_none() {
            self.span = Some(span.span());
        }

        self
    }

    pub const fn inner(&self) -> &T { &self.inner }

    pub fn inner_mut(&mut self) -> &mut T { &mut self.inner }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            inner: f(self.inner),
            span: self.span,
        }
    }

    pub fn into_inner(self) -> T { self.inner }
}

impl<T> From<T> for Spanned<T> {
    fn from(inner: T) -> Self { Self::new(inner) }
}

pub(crate) trait AttributeExt {
    type Target: Sized;

    fn from_token_stream(_: TokenStream) -> syn::Result<Self::Target>;

    fn from_str(_: &str) -> syn::Result<Self::Target>;
}

pub(crate) trait PathExt {
    fn to_string(&self) -> String;
}

pub(crate) trait MetaExt {
    fn to_string(&self) -> String;
}

impl AttributeExt for Attribute {
    type Target = Self;

    fn from_token_stream(input: TokenStream) -> syn::Result<Self::Target> {
        let parser = Self::parse_outer;

        Ok(parser.parse2(input)?.into_iter().next().unwrap())
    }

    fn from_str(input: &str) -> syn::Result<Self::Target> {
        let parser = Self::parse_outer;

        Ok(parser.parse_str(input)?.into_iter().next().unwrap())
    }
}

pub(crate) trait ErrorExt {
    fn multiple<T>(errors: T) -> syn::Error
    where
        T: IntoIterator<Item = syn::Error>;
}

impl ErrorExt for syn::Error {
    fn multiple<T>(errors: T) -> syn::Error
    where
        T: IntoIterator<Item = syn::Error>,
    {
        let mut errors = errors.into_iter();
        let mut result = errors
            .next()
            .expect("failed to create an Error from an empty Iterator");

        for error in errors {
            result.combine(error);
        }

        result
    }
}

impl MetaExt for Meta {
    fn to_string(&self) -> String { self.path().to_string() }
}

impl PathExt for Path {
    fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<String>>()
            .join("::")
    }
}

pub(crate) trait TypeExt {
    fn path(&self) -> Option<&Path>;

    fn arguments(&self) -> Option<Vec<TokenStream>>;

    fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>;

    fn is_primitive_copy(&self) -> bool;

    fn is_option(&self) -> bool;

    fn to_as_ref(&self) -> Option<TokenStream>;

    fn is_reference(&self) -> bool;
}

impl TypeExt for Type {
    fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>,
    {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            last.ident == *ident
        } else {
            false
        }
    }

    fn path(&self) -> Option<&Path> {
        match &self {
            Self::Path(ty) => Some(&ty.path),
            _ => None,
        }
    }

    fn arguments(&self) -> Option<Vec<TokenStream>> {
        let path = self.path()?;

        if let Some(last) = path.segments.last() {
            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => {
                    return Some(bracketed.args.iter().map(|v| quote!(#v)).collect())
                }
                _ => return None,
            }
        }

        None
    }

    fn is_reference(&self) -> bool {
        match &self {
            Self::Reference(_) => true,
            _ => false,
        }
    }

    fn is_primitive_copy(&self) -> bool {
        let path = match &self {
            // Array types of all sizes implement copy, if the item type implements copy.
            Self::Array(ty) => return ty.elem.is_primitive_copy(),
            Self::Group(ty) => return ty.elem.is_primitive_copy(),
            Self::Paren(ty) => return ty.elem.is_primitive_copy(),
            Self::Path(ty) => &ty.path,
            // mutable references do not implement copy:
            Self::Reference(ty) => return ty.mutability.is_none(),
            Self::Tuple(ty) => return !ty.elems.iter().map(|s| s.is_primitive_copy()).any(|e| !e),
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            match last.ident.to_string().as_ref() {
                "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => return true,
                _ => return false,
            }
        }

        false
    }

    fn is_option(&self) -> bool {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            if last.ident != "Option" {
                return false;
            }

            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => return bracketed.args.len() == 1,
                _ => return false,
            }
        }

        false
    }

    fn to_as_ref(&self) -> Option<TokenStream> {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return None,
        };

        if let Some(last) = path.segments.last() {
            if last.ident != "Option" {
                return None;
            }

            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => {
                    let args = &bracketed.args;
                    let ident = &last.ident;
                    return Some(quote!(#ident<&#args>));
                }
                _ => return None,
            }
        }

        None
    }
}
