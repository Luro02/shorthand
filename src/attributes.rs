use std::collections::HashMap;

use darling::Error;
use from_map::FromMap;
use syn::{Meta, NestedMeta};

use crate::utils::MetaExt;

#[derive(Default)]
pub(crate) struct AttributesBuilder {
    fields: HashMap<&'static str, bool>,
    errors: Vec<Error>,
}

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
                                self.errors.push(
                                    Error::custom("Unexpected field `rename`")
                                        .with_span(&inner)
                                        .at(&ident),
                                );
                            } else {
                                insert_field = Some("rename");
                            }
                            unknown = false;
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
                            if self.fields.insert(field, state).is_some()
                                && !(field == "forward_attributes" || field == "forward_everything")
                            {
                                // error if insert was already called -> duplicate field
                                self.errors.push(
                                    Error::duplicate_field(field).with_span(&inner).at(&ident),
                                );
                            }
                        }

                        if unknown {
                            self.errors.push(
                                Error::unknown_field_with_alts(name.as_str(), &Self::FIELDS)
                                    .with_span(&inner)
                                    .at(&ident),
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
        }

        self
    }

    pub fn build_with(self, mut result: Attributes) -> Result<Attributes, Error> {
        if !self.errors.is_empty() {
            return Err(Error::multiple(self.errors));
        }

        result.with_map(&self.fields);

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
    pub fn builder() -> AttributesBuilder {
        AttributesBuilder::default()
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
