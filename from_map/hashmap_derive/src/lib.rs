//! This crate only exists, because it is not possible to export a trait and a
//! `proc_macro` in the same crate.
//!
//! You should instead use the [`from_map`](https://crates.io/crates/from_map) crate.
#![warn(clippy::pedantic, clippy::nursery)]
extern crate proc_macro;

mod expand;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(FromMap)]
pub fn from_hashmap(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    expand::derive(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
