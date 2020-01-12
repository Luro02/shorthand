use std::collections::HashMap;

use from_map::FromMap;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Meta, NestedMeta};

use crate::error::Error;
use crate::utils::MetaExt;

#[derive(Default)]
pub(crate) struct AttributesBuilder {
    fields: HashMap<&'static str, (bool, Span)>,
    errors: Vec<Error>,
}

/*
What should trigger an error (redundant attributes):
- enabling an attribute, that is already enabled
- disabling an attribute, that is already disabled
- enabling/disabling an attribute multiple times
  #[shorthand(enable(const_fn, const_fn))]
*/

impl AttributesBuilder {
    const FIELDS: [&'static str; 18] = [
        "option_as_ref",
        "const_fn",
        "primitive_copy",
        "inline",
        "must_use", // TODO: this should only work on getter and mut getter
        "copy",
        "get",
        "set",
        "ignore_phantomdata",
        "skip",
        "rename",
        "into",
        "forward_attributes",
        "forward_everything",
        "ignore_underscore",
        "try_into",
        "get_mut",
        "collection_magic",
    ];

    pub fn push_meta(&mut self, ident: &str, item: &Meta) -> &mut Self {
        let state = {
            match ident {
                "enable" => true,
                "disable" => false,
                _ => {
                    unreachable!("ident is neither `enable` nor `disable`");
                }
            }
        };

        if let Meta::List(value) = &item {
            for nested_item in &value.nested {
                match &nested_item {
                    NestedMeta::Meta(inner) => {
                        let name = inner.to_string();
                        let mut unknown = true;
                        let mut insert_field: Option<&'static str> = None;

                        if name == "rename" {
                            // #[shorthand(enable(rename))] is invalid
                            if state {
                                self.errors
                                    .push(Error::unexpected_field(&name).with_span(&inner));
                            } else {
                                insert_field = Some("rename");
                            }
                            unknown = false;
                        // forward will be parse from `Options` directly,
                        // therefore this field should
                        // be ignored.
                        } else if name == "forward" {
                            return self;
                        } else {
                            for field in &Self::FIELDS {
                                if &name == field {
                                    insert_field = Some(field);
                                    unknown = false;
                                    break;
                                }
                            }
                        }

                        if let Some(field) = insert_field {
                            // error for this invariant:
                            // #[shorthand(enable(const_fn, const_fn))]
                            if self.fields.insert(field, (state, inner.span())).is_some()
                                && !(field == "forward_attributes" || field == "forward_everything")
                            {
                                // error if insert was already called -> duplicate field
                                self.errors
                                    .push(Error::duplicate_field(field).with_span(&inner));
                            }
                        }

                        if unknown {
                            self.errors.push(
                                Error::unknown_field(name.as_str())
                                    .with_alts(&Self::FIELDS)
                                    .with_span(&inner),
                            );
                        }
                    }
                    // Error for `#[shorthand(enable("option_as_ref"))]`
                    NestedMeta::Lit(lit) => {
                        self.errors
                            .push(Error::unexpected_lit_type(lit).with_span(nested_item));
                    }
                }
            }
        } else {
            // TODO: error
        }

        self
    }

    pub fn build_with(mut self, mut result: Attributes) -> Result<Attributes, Error> {
        // loop through all fields of result:
        for (k, v) in result.as_map() {
            if k == "forward_attributes" || k == "forward_everything" {
                continue;
            }
            if let Some(value) = self.fields.get(&k) {
                if v == value.0 {
                    let state = {
                        if v {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    };

                    self.errors
                        .push(Error::redundant_field(k, Some(state)).with_span(&value.1));
                }
            }
        }

        result.with_map(&self.fields.into_iter().map(|(k, v)| (k, v.0)).collect());

        if !self.errors.is_empty() {
            return Err(Error::multiple(self.errors));
        }

        Ok(result)
    }

    // pub fn build(self) -> Result<Attributes, Error> {
    //     self.build_with(Attributes::default())
    // }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, FromMap)]
pub(crate) struct Attributes {
    pub option_as_ref: bool,
    pub const_fn: bool,
    pub primitive_copy: bool,
    pub inline: bool,
    pub must_use: bool,
    pub copy: bool,
    pub get: bool,
    pub set: bool,
    pub ignore_phantomdata: bool,
    pub skip: bool,
    // `true` means that the field can be renamed
    pub rename: bool,
    pub into: bool,
    pub forward_attributes: bool,
    pub forward_everything: bool,
    pub ignore_underscore: bool,
    pub try_into: bool,
    pub get_mut: bool,
    pub collection_magic: bool,
}

impl Attributes {
    pub fn builder() -> AttributesBuilder { AttributesBuilder::default() }

    pub fn with_meta(result: Self, ident: &str, item: &Meta) -> Result<Self, Error> {
        let mut builder = Self::builder();

        builder.push_meta(ident, item);

        builder.build_with(result)
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            option_as_ref: true,
            const_fn: false,
            primitive_copy: true,
            inline: true,
            must_use: false,
            copy: false,
            get: true,
            set: true,
            ignore_phantomdata: true,
            skip: false,
            rename: true,
            into: false,
            forward_attributes: true,
            forward_everything: false,
            ignore_underscore: false,
            try_into: false,
            get_mut: false,
            collection_magic: false,
        }
    }
}
