use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Attribute, Meta, Path, PathArguments, Type};

pub(crate) trait AttributeExt {
    type Target: Sized;

    fn from_token_stream(_: TokenStream) -> syn::Result<Self::Target>;
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

        Ok(parser.parse2(input)?.into_iter().nth(0).unwrap())
    }
}

impl MetaExt for Meta {
    fn to_string(&self) -> String {
        self.path().to_string()
    }
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

    fn inner_type(&self) -> Option<TokenStream>;

    fn is_phantom_data(&self) -> bool;

    fn is_vec(&self) -> bool;

    fn is_primitive_copy(&self) -> bool;

    fn is_option(&self) -> bool;

    fn to_as_ref(&self) -> Option<TokenStream>;
}

impl TypeExt for Type {
    fn path(&self) -> Option<&Path> {
        match &self {
            Self::Path(ty) => Some(&ty.path),
            _ => None,
        }
    }

    fn inner_type(&self) -> Option<TokenStream> {
        let path = self.path()?;

        if let Some(last) = path.segments.last() {
            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => {
                    let args = &bracketed.args;
                    return Some(quote!(#args));
                }
                _ => return None,
            }
        }

        None
    }

    fn is_phantom_data(&self) -> bool {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            if last.ident == "PhantomData" {
                return true;
            }
        }

        false
    }

    fn is_vec(&self) -> bool {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            if last.ident == "Vec" {
                return true;
            }
        }

        false
    }

    fn is_primitive_copy(&self) -> bool {
        let path = match &self {
            Self::Path(ty) => &ty.path,
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
