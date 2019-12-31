use indexmap::{IndexMap, IndexSet};

use darling::{Error, FromDeriveInput, FromGenerics, FromMeta};
use syn::{Data, DeriveInput, Generics, Ident, Lit, LitStr, Meta, NestedMeta, Visibility};

use crate::attributes::Attributes;
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

/* Current design:
Options struct, that contains all the data
*/

/*
struct ShortHandAttribute {
    pound_token: Token![#],
    bracket_token: token::Bracket,
    shorthand_token: kw::shorthand,
}

struct Rename {
    content: TokenStream,
    //
    pub enabled: bool,
    pub format: String,
}

mod kw {
    syn::custom_keyword!(shorthand);
}

impl Parse for Rename {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // #[shorthand(rename("prefix_{}_suffix"))]
        // #[shorthand(rename(format = "", get = "", set = "", get_mut = "", try_set = ""))]
        let content;
        Ok(Self {
            pound_token: input.parse()?,
            bracket_token: bracketed!(content in input),
            content: content.parse()?,
        })
    }
}*/

#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub ident: Ident,
    pub attrs: IndexSet<syn::Attribute>,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: Data,

    pub visibility: Visibility,
    pub attributes: Attributes,
    pub get_format: Option<String>,
    pub set_format: Option<String>,
    pub try_set_format: Option<String>,
    pub get_mut_format: Option<String>,
}

fn parse_rename_attribute(
    values: Vec<Meta>,
    errors: &mut Vec<Error>,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut get_format = None;
    let mut set_format = None;
    let mut try_set_format = None;
    let mut get_mut_format = None;

    for value in values {
        match value {
            // #[shorthand(rename)]
            Meta::Path(path) => {
                errors.push(Error::custom("Unexpected field `rename`").with_span(&path));
            }
            Meta::List(meta_list) => {
                for attr in meta_list.nested {
                    match attr {
                        // #[shorthand(rename("prefix_{}_suffix"))]
                        NestedMeta::Lit(lit) => {
                            if let Lit::Str(lit_str) = lit {
                                set_format = Some((true, lit_str.clone()));
                                try_set_format = Some((true, lit_str.clone()));
                                get_mut_format = Some((true, lit_str.clone()));
                                get_format = Some(lit_str);
                                break;
                            } else {
                                // #[shorthand(rename(b"{}"))]
                                // #[shorthand(rename(0))]
                                errors.push(Error::unexpected_lit_type(&lit).with_span(&lit));
                            }
                        }
                        // #[shorthand(rename(format = "prefix_{}_suffix"))]
                        NestedMeta::Meta(meta) => {
                            if let Meta::NameValue(pair) = meta {
                                if let Lit::Str(lit_str) = pair.lit {
                                    match pair.path.to_string().as_str() {
                                        // rename(format = "prefix_{}_suffix")
                                        "format" => {
                                            set_format = Some((true, lit_str.clone()));
                                            try_set_format = Some((true, lit_str.clone()));
                                            get_mut_format = Some((true, lit_str.clone()));
                                            get_format = Some(lit_str);
                                        }
                                        // rename(get = "prefix_{}_suffix")
                                        "get" => {
                                            get_format = Some(lit_str);
                                        }
                                        // rename(get = "prefix_{}_suffix")
                                        "set" => {
                                            set_format = Some((false, lit_str));
                                        }
                                        "try_set" => {
                                            try_set_format = Some((false, lit_str));
                                        }
                                        "get_mut" => {
                                            get_mut_format = Some((false, lit_str));
                                        }
                                        // rename(garbage = "prefix_{}_suffix")
                                        _ => {
                                            errors.push(
                                                Error::unknown_field_with_alts(
                                                    &pair.path.to_string(),
                                                    &["format", "get", "set", "try_set", "get_mut"],
                                                )
                                                .with_span(&pair.path),
                                            );
                                        }
                                    }
                                // rename(get = b"prefix_{}_suffix")
                                } else {
                                    errors.push(
                                        Error::unexpected_lit_type(&pair.lit).with_span(&pair.lit),
                                    );
                                }
                            // #[shorthand(rename(format))]
                            // or
                            // #[shorthand(rename(format(xyz)))]
                            } else {
                                let format = {
                                    match meta {
                                        Meta::Path(_) => "Path",
                                        Meta::List(_) => "List",
                                        _ => "",
                                    }
                                };

                                errors.push(Error::unsupported_format(format).with_span(&meta));
                            }
                        }
                    }
                }
            }
            // #[shorthand(rename = "example")]
            Meta::NameValue(name_value) => {
                errors.push(Error::custom("Unsupported format").with_span(&name_value));
            }
        }
    }

    if let Some(value) = &mut set_format {
        if value.0 {
            value.1 = LitStr::new(
                {
                    let mut result = String::from("set_");
                    result.push_str(&value.1.value());
                    result
                }
                .as_str(),
                value.1.span(),
            );
        }
    }

    if let Some(value) = &mut try_set_format {
        if value.0 {
            value.1 = LitStr::new(
                {
                    let mut result = String::from("try_");
                    result.push_str(&value.1.value());
                    result
                }
                .as_str(),
                value.1.span(),
            );
        }
    }

    if let Some(value) = &mut get_mut_format {
        if value.0 {
            value.1 = LitStr::new(
                {
                    let mut result = value.1.value();
                    result.push_str("_mut");
                    result
                }
                .as_str(),
                value.1.span(),
            );
        }
    }

    for lit_str in get_format
        .iter()
        .chain(set_format.iter().map(|s| &s.1))
        .chain(try_set_format.iter().map(|s| &s.1))
    {
        match lit_str.value().matches("{}").count() {
            0 => {
                errors.push(Error::custom("Missing `{}` in format string.").with_span(&lit_str));
            }
            1 => {}
            _ => {
                errors.push(
                    Error::custom("More than one `{}` in format string.").with_span(&lit_str),
                );
            }
        }
    }

    (
        get_format.map(|s| s.value()),
        set_format.map(|s| s.1.value()),
        try_set_format.map(|s| s.1.value()),
        get_mut_format.map(|s| s.1.value()),
    )
}

impl Options {
    const FIELDS: [&'static str; 4] = ["enable", "disable", "visibility", "rename"];

    fn parse_attributes<T>(mut result: Self, attrs: &T) -> Result<Self, Error>
    where
        for<'a> &'a T: IntoIterator<Item = &'a syn::Attribute>,
    {
        // there is a list of errors, so you can see more than one compiler error
        // and don't have to recompile the entire codebase, just to see the next error...
        let mut errors = Vec::new();
        // An IndexMap is like a HashMap except, that it does preserve the order, in which
        // the elements were inserted.
        // This is important in order to test the error output of this macro.
        // Otherwise some tests might fail randomly, because the order of the error messages
        // might have changed!
        let mut fields = IndexMap::<&'static str, Vec<_>>::new();

        // dbg!(attrs
        //     .into_iter()
        //     .cloned()
        //     .map(|s| s.path.to_string())
        //     .collect::<Vec<_>>());

        // iterate through all attributes
        let mut forward_attributes = result.attributes.forward_attributes;

        for attr in attrs {
            println!("state: {}", forward_attributes);
            println!("path: {}", attr.path.to_string());

            match attr.path.to_string().as_str() {
                "shorthand" => {
                    if let Ok(Meta::List(data)) = attr.parse_meta() {
                        for item in &data.nested {
                            if let NestedMeta::Meta(inner) = &item {
                                // name is for ex. `enable` or `disable`
                                let name = inner.to_string();
                                // this flag will check for any unknown fields
                                let mut unknown = true;

                                // All known fields are in `Self::FIELDS`, this
                                // makes it easier to add new fields.
                                for field in &Self::FIELDS {
                                    if &name == field {
                                        // Self::FIELDS are allowed to be seen multiple times so,
                                        // check wether a field has already been seen:
                                        if let Some(values) = fields.get_mut(field) {
                                            values.push(inner.clone());
                                        } else {
                                            fields.insert(field, vec![inner.clone()]);
                                        }

                                        unknown = false;
                                        break;
                                    }
                                }

                                // If the field is `unknown` add it to the list of `errors`:
                                if unknown {
                                    errors.push(
                                        Error::unknown_field_with_alts(
                                            name.as_str(),
                                            &Self::FIELDS,
                                        )
                                        .with_span(&inner)
                                        .at(&name),
                                    );
                                }
                            }
                        }
                    } else {
                        continue;
                    }
                }
                // All built-in attributes will be forwarded (the ones that are for functions):
                // based on: https://doc.rust-lang.org/reference/attributes.html
                //
                // Documentation
                "doc"
                // Conditional compilation
                | "cfg" | "cfg_attr"
                // Diagnostic attributes
                | "allow" | "warn" | "deny" | "forbid"
                | "deprecated" | "must_use"
                // Code generation
                | "inline"
                | "cold"
                | "target_feature"
                => {
                    if result.attributes.forward_attributes {
                        result.attrs.insert(attr.clone());
                    }
                },
                // the rest is ignored, to prevent conflicts with other macros
                _ => {
                    if result.attributes.forward_everything
                        && result.attributes.forward_attributes
                    {
                        result.attrs.insert(attr.clone());
                    }
                    continue;
                },
            }
        }

        let mut attributes = Attributes::builder();

        for (key, values) in fields {
            match key {
                "enable" | "disable" => {
                    for value in values {
                        attributes.push_meta(key, &value);
                    }
                }
                "visibility" => {
                    // TODO: does this allow multiple visibility to overwrite each other?
                    for value in values {
                        match FieldVisibility::from_meta(&value)
                            .map_err(|e| e.with_span(&value).at(key))
                        {
                            Ok(val) => {
                                result.visibility = val.into_inner();
                            }
                            Err(e) => {
                                errors.push(e);
                            }
                        };
                    }
                }
                "rename" => {
                    // TODO: this might cause problems, if someone tries
                    // #[shorthand(rename(...))]
                    // #[shorthand(rename(...))]
                    // but I think it's okay to silently overwrite
                    let (get_format, set_format, try_set_format, get_mut_format) =
                        parse_rename_attribute(values, &mut errors);
                    result.get_format = get_format;
                    result.set_format = set_format;
                    result.try_set_format = try_set_format;
                    result.get_mut_format = get_mut_format;
                }
                _ => {}
            }
        }

        match attributes.build_with(result.attributes) {
            Ok(value) => {
                result.attributes = value;
            }
            Err(e) => {
                errors.push(e);
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

    pub const fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

impl FromDeriveInput for Options {
    fn from_derive_input(input: &DeriveInput) -> Result<Self, Error> {
        let result = Self {
            ident: input.ident.clone(),
            generics: FromGenerics::from_generics(&input.generics)?,
            vis: input.vis.clone(),
            attrs: IndexSet::new(),
            data: input.data.clone(),

            visibility: FieldVisibility::default().into_inner(),
            attributes: Attributes::default(),
            get_format: None,
            set_format: None,
            try_set_format: None,
            get_mut_format: None,
        };

        Ok(Self::parse_attributes(result, &input.attrs)?)
    }
}
