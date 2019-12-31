use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

pub fn derive(input: &DeriveInput) -> syn::Result<TokenStream> {
    let mut errors = vec![];
    let mut types = vec![];

    let idents = {
        let mut result = vec![];

        match &input.data {
            Data::Struct(s) => match &s.fields {
                Fields::Named(named) => {
                    for field in &named.named {
                        if let Some(ident) = &field.ident {
                            result.push(ident.clone());
                            types.push(field.ty.clone());
                        } else {
                            unreachable!("field has no name");
                        }
                    }
                }
                _ => {
                    errors.push(Error::new_spanned(
                        &s.fields,
                        "Unnamed Fields are not supported.",
                    ));
                }
            },
            _ => {
                errors.push(Error::new_spanned(
                    &input.ident,
                    "Enums/Unions are not supported.",
                ));
            }
        }

        result
    };

    if !errors.is_empty() {
        let mut iterator = errors.into_iter();
        let mut main_error = iterator.next().unwrap();

        for error in iterator {
            main_error.combine(error);
        }

        return Err(main_error);
    }

    let value = types.first().unwrap(); // TODO
    let keys = idents.iter().map(|i| i.to_string()).collect::<Vec<_>>();
    let name = &input.ident;

    Ok(quote! {
        impl FromMap for #name {
            type Value = #value;

            fn with_map(
                &mut self,
                input: &::std::collections::HashMap<&'static str, Self::Value>
            ) {
                #(
                    match input.get(#keys) {
                        Some(value) => {
                            self.#idents = value.clone();
                        },
                        _ => {},
                    }
                )*
            }
        }
    })
}
