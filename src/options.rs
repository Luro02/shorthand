use darling::{Error, FromDeriveInput, FromGenerics};
use quote::quote;
use syn::{Data, DeriveInput, Generics, Ident, Meta, NestedMeta, Visibility};

use crate::attributes::Attributes;
use crate::forward::Forward;
use crate::rename::Rename;
use crate::utils::{MetaExt, PathExt};
use crate::visibility::FieldVisibility;

/*
Attributes, that should be supported:

# done:
- option_as_ref
- copy
- const_fn
- ignore_phantomdata
- get
- set
- skip
- rename
- rename("get_{}_value")
- into
- primitive_copy
- ignore_underscore (ignores fields with an ident, that starts with an underscore)
- get_mut
- try_into

# todo:
// TODO: serde seems to have something similar (maybe use this as reference?)
- custom(generic = "T: Read", return = "K", func = "path::to::function")
- bounds(with_func = "path::to::function") // see serde for example
- collection_magic
- collection_insert
- collection_push
- cow
- as_ref
- use_naming_convention (getter will return `as_...`), functions that return bool will be `is_...`

# How to specify attributes:
#[shorthand(visibility(pub), enable(option_as_ref, const_fn, copy_primitive), disable(cow, set))]
*/

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
}

impl Options {
    const FIELDS: [&'static str; 4] = ["enable", "disable", "visibility", "rename"];

    fn parse_attributes<T>(mut result: Self, attrs: &T) -> Result<Self, Error>
    where
        for<'a> &'a T: IntoIterator<Item = &'a syn::Attribute>,
    {
        // there is a list of errors, so you can see more than one compiler error
        // and don't have to recompile the entire codebase, just to see the next
        // error...
        let mut errors = Vec::new();
        // This HashSet is used to see, which attributes have already been inserted.
        // This is specifically for the case where one tries to do
        //
        // ```
        // #[shorthand(enable(attribute))]
        // #[shorthand(disable(attribute))]
        // field: String,
        // ```
        //
        // As you can see this makes no sense and therefore this library should error,
        // if an attribute is encountered multiple times!
        // let mut visited = HashSet::new();

        // iterate through all attributes
        for attr in attrs {
            let meta = {
                match attr.parse_meta() {
                    Ok(val) => val,
                    Err(e) => {
                        errors.push(Error::custom(&e).with_span(&e.span()));
                        continue;
                    }
                }
            };

            if Forward::is_forward(&meta) {
                let forward = result.forward.update(&{
                    match syn::parse2(quote!(#attr)) {
                        Ok(val) => val,
                        Err(e) => {
                            errors.push(Error::custom(&e).with_span(&e.span()));
                            return Err(Error::multiple(errors));
                        }
                    }
                });

                result.forward = forward;

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
                            // TOOD: this loop makes little to no sense right now!
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
                                                result.visibility = value.into_inner();
                                            }
                                            Err(e) => {
                                                errors.push(
                                                    Error::custom(e).with_span(&inner).at(field),
                                                );
                                            }
                                        }
                                    } else if field == &"rename" {
                                        match syn::parse2(quote!(#attr)) {
                                            Ok(attr) => {
                                                result.rename = attr;
                                            }
                                            Err(err) => {
                                                errors.push(
                                                    darling::Error::custom(&err)
                                                        .with_span(&err.span()),
                                                );
                                            }
                                        }
                                    } else {
                                        unreachable!(format!("Unhandled attribute: {}", field));
                                    }
                                    unknown = false;
                                    break;
                                }
                            }

                            // If the field is `unknown` add it to the list of `errors`:
                            if unknown {
                                errors.push(
                                    Error::unknown_field_with_alts(name.as_str(), &Self::FIELDS)
                                        .with_span(&inner)
                                        .at(&name),
                                );
                            }
                        }
                    }
                } else {
                    // TODO: error?
                    continue;
                }
            } else {
                if result.forward.is(attr.path.to_string().as_str()) {
                    result.attrs.push(attr.clone());
                }
                continue;
            }
        }

        if !errors.is_empty() {
            return Err(Error::multiple(errors));
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

    pub const fn visibility(&self) -> &Visibility { &self.visibility }
}

impl FromDeriveInput for Options {
    fn from_derive_input(input: &DeriveInput) -> Result<Self, Error> {
        let result = Self {
            ident: input.ident.clone(),
            generics: FromGenerics::from_generics(&input.generics)?,
            vis: input.vis.clone(),
            attrs: Vec::new(),
            data: input.data.clone(),
            forward: Forward::default(),

            visibility: FieldVisibility::default().into_inner(),
            attributes: Attributes::default(),
            rename: Rename::default(),
        };

        Ok(Self::parse_attributes(result, &input.attrs)?)
    }
}
