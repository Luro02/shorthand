use quote::quote;
use syn::{Data, DeriveInput, Generics, Ident, Meta, NestedMeta, Visibility};

use crate::attributes::Attributes;
use crate::error::Error;
use crate::forward::Forward;
use crate::rename::Rename;
use crate::utils::{MetaExt, PathExt};
use crate::verify::Verify;
use crate::visibility::FieldVisibility;

#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub ident: Ident,
    pub attrs: Vec<syn::Attribute>,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: Data,
    forward: Forward,

    pub visibility: Visibility,
    pub attributes: Attributes,
    pub rename: Rename,
    pub verify: Verify,
    is_initial: bool,
}

impl Options {
    const FIELDS: [&'static str; 5] = ["enable", "disable", "visibility", "rename", "verify"];

    fn parse_attributes<T>(mut result: Self, attrs: &T) -> Result<Self, Error>
    where
        for<'a> &'a T: IntoIterator<Item = &'a syn::Attribute>,
    {
        // there is a list of errors, so you can see more than one compiler error
        // and don't have to recompile the entire codebase, just to see the next
        // error...
        let mut errors = Vec::new();

        // iterate through all attributes
        for attr in attrs {
            let meta = {
                match attr.parse_meta() {
                    Ok(val) => val,
                    Err(e) => {
                        errors.push(Error::syn(e));
                        continue;
                    }
                }
            };

            if Forward::is_forward(&meta) {
                result.forward.update({
                    match syn::parse2(quote!(#attr)) {
                        Ok(val) => val,
                        Err(e) => {
                            errors.push(Error::syn(e));
                            continue;
                        }
                    }
                });

                continue;
            }

            if let "shorthand" = attr.path.to_string().as_str() {
                if let Meta::List(data) = meta {
                    for item in &data.nested {
                        if let NestedMeta::Meta(inner) = &item {
                            // name is for ex. `enable` or `disable`
                            let name = inner.to_string();
                            // this flag will check for any unknown fields
                            let mut unknown = true;

                            // All known fields are in `Self::FIELDS`, this
                            // makes it easier to add new fields.
                            // TODO: remove the loop?
                            for field in &Self::FIELDS {
                                if &name == field {
                                    if field == &"enable" || field == &"disable" {
                                        match Attributes::with_meta(result.attributes, field, inner)
                                        {
                                            Ok(val) => {
                                                result.attributes = val;
                                            }
                                            Err(err) => {
                                                errors.push(err);
                                            }
                                        }
                                    } else if field == &"visibility" {
                                        match syn::parse2::<FieldVisibility>(quote!(#attr)) {
                                            Ok(value) => {
                                                result.visibility = value
                                                    .into_inner()
                                                    .unwrap_or_else(|| result.vis.clone());
                                            }
                                            Err(err) => {
                                                errors.push(Error::syn(err));
                                            }
                                        }
                                    } else if field == &"rename" {
                                        match syn::parse2(quote!(#attr)) {
                                            Ok(attr) => {
                                                result.rename = attr;
                                            }
                                            Err(err) => {
                                                errors.push(Error::syn(err));
                                            }
                                        }
                                    } else if field == &"verify" {
                                        match syn::parse2(quote!(#attr)) {
                                            Ok(attr) => {
                                                result.verify = attr;
                                            }
                                            Err(err) => {
                                                errors.push(Error::syn(err));
                                            }
                                        }
                                    } else {
                                        unreachable!(format!("unhandled field: {}", field));
                                    }

                                    unknown = false;
                                    break;
                                }
                            }

                            // If the field is `unknown` add it to the list of `errors`:
                            if unknown {
                                errors.push(
                                    Error::unknown_field(name.as_str())
                                        .with_alts(&Self::FIELDS)
                                        .with_span(&inner),
                                );
                            }
                        }
                    }
                } else {
                    errors.push(Error::unexpected_meta(&meta).with_alts(&["List"]));
                    continue;
                }
            } else {
                if result.forward.is(attr.path.to_string().as_str()) && !result.is_initial {
                    result.attrs.push(attr.clone());
                }
                continue;
            }
        }

        if !errors.is_empty() {
            return Err(Error::multiple(errors));
        }

        if result.is_initial {
            result.is_initial = false;
        }

        Ok(result)
    }

    pub fn with_attrs<T>(&self, attrs: &T) -> Result<Self, Error>
    where
        for<'a> &'a T: IntoIterator<Item = &'a syn::Attribute>,
    {
        let result = self.clone();
        Ok(Self::parse_attributes(result, attrs)?)
    }
}

impl Options {
    pub fn from_derive_input(input: &DeriveInput) -> Result<Self, Error> {
        let result = Self {
            ident: input.ident.clone(),
            generics: input.generics.clone(),
            vis: input.vis.clone(),
            attrs: Vec::new(),
            data: input.data.clone(),
            forward: Forward::default(),

            visibility: FieldVisibility::default()
                .into_inner()
                .unwrap_or_else(|| input.vis.clone()),
            attributes: Attributes::default(),
            rename: Rename::default(),
            verify: Verify::default(),
            is_initial: true,
        };

        Ok(Self::parse_attributes(result, &input.attrs)?)
    }
}
