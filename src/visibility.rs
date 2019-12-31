//! This module is for the visibility attribute

use darling::{Error, FromMeta};
use quote::quote;
use syn::{NestedMeta, Visibility};

#[derive(Debug, Clone)]
pub(crate) struct FieldVisibility {
    pub visibility: Visibility,
}

impl FieldVisibility {
    pub fn into_inner(self) -> Visibility {
        self.visibility
    }
}

impl FromMeta for FieldVisibility {
    fn from_list(items: &[NestedMeta]) -> Result<Self, Error> {
        if items.is_empty() {
            return Err(Error::too_few_items(1));
        } else if items.len() > 1 {
            return Err(Error::too_many_items(1));
        }

        let item = &items[0];

        Ok(Self {
            // TODO: support the visibility for getters, setters and mutgetters
            visibility: syn::parse2(quote!(#item)).map_err(|_| {
                Error::unknown_field_with_alts(
                    &quote!(#item).to_string(),
                    &[
                        "pub",
                        "pub(crate)",
                        "pub(self)",
                        "pub(super)",
                        "pub(in ::path::to::mod)",
                    ],
                )
            })?,
        })
    }
}

impl Default for FieldVisibility {
    fn default() -> Self {
        Self {
            visibility: syn::parse("pub".parse().unwrap()).unwrap(),
        }
    }
}
