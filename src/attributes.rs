use std::collections::HashMap;

use from_map::FromMap;
use syn::spanned::Spanned as _;
use syn::{Meta, NestedMeta};

use crate::error::Error;
use crate::utils::MetaExt;
use crate::utils::Spanned;

#[derive(Default)]
pub(crate) struct AttributesBuilder {
    fields: HashMap<Spanned<&'static str>, bool>,
    errors: Vec<Error>,
}

/*
What should trigger redundant error:
- enabling an attribute, that is already enabled
- disabling an attribute, that is already disabled
- enabling/disabling an attribute multiple times
  #[shorthand(enable(const_fn, const_fn))]
*/

impl AttributesBuilder {
    const FIELDS: [&'static str; 19] = [
        "option_as_ref",
        "const_fn",
        "primitive_copy",
        "inline",
        "must_use",
        "copy",
        "get",
        "set",
        "into",
        "try_into",
        "get_mut",
        "ignore_phantomdata",
        "skip",
        "rename",
        "forward",
        "ignore_underscore",
        "collection_magic",
        "strip_option",
        "clone",
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
                            if self
                                .fields
                                .insert(Spanned::new(field).with_span(&inner.span()), state)
                                .is_some()
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
                            .push(Error::unexpected_lit(lit).with_span(nested_item));
                    }
                }
            }
        } else {
            self.errors.push(Error::unexpected_meta(item));
        }

        self
    }

    pub fn build_with(mut self, mut result: Attributes) -> Result<Attributes, Error> {
        // loop through all fields of result:
        for (field, v) in result.as_map() {
            // TODO: this is temporary and should be removed as soon as those attributes
            //       are replaced by "forward"
            if field == "forward_attributes" || field == "forward_everything" {
                continue;
            }

            if let Some((field, value)) = self.fields.get_key_value(&Spanned::new(field)) {
                if v == *value {
                    let state = {
                        if v {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    };

                    self.errors.push(
                        Error::redundant_field(*field.inner(), Some(state))
                            .with_span(&field.span()),
                    );
                }
            }
        }

        result.with_map(
            &self
                .fields
                .into_iter()
                .map(|(k, v)| (k.into_inner(), v))
                .collect(),
        );

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
    pub ignore_underscore: bool,
    pub try_into: bool,
    pub get_mut: bool,
    pub collection_magic: bool,
    pub strip_option: bool,
    pub clone: bool,
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
            ignore_underscore: false,
            try_into: false,
            get_mut: false,
            collection_magic: false,
            strip_option: false,
            clone: false,
        }
    }
}
