use core::fmt;

use proc_macro2::TokenStream;

#[derive(Debug)]
pub struct Error {
    internal: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    DarlingError(darling::Error),
    SynError(syn::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.internal {
            ErrorKind::DarlingError(e) => write!(f, "{}", e),
            ErrorKind::SynError(e) => write!(f, "{}", e),
        }
    }
}

impl From<darling::Error> for Error {
    fn from(value: darling::Error) -> Self { Self::darling(value) }
}

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self { Self::syn(value) }
}

impl Error {
    pub fn darling(value: darling::Error) -> Self {
        Self {
            internal: ErrorKind::DarlingError(value),
        }
    }

    pub fn syn(value: syn::Error) -> Self {
        Self {
            internal: ErrorKind::SynError(value),
        }
    }

    pub fn into_token_stream(self) -> TokenStream {
        match self.internal {
            ErrorKind::DarlingError(e) => e.write_errors(),
            ErrorKind::SynError(e) => e.to_compile_error(),
        }
    }
}
