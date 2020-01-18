use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned as _;
use syn::{Attribute, Data, DeriveInput, Field, Fields, Ident, Type};

use crate::error::Error;
use crate::options::Options;
use crate::utils::{AttributeExt, PathExt, TypeExt};

pub fn derive(input: &DeriveInput) -> crate::Result<TokenStream> {
    let name = &input.ident;
    let mut errors = vec![];

    let mut functions: Vec<TokenStream> = vec![];

    let options = Options::from_derive_input(input)?;

    match &options.data {
        Data::Struct(s) => {
            let generator = Generator::from_options(&options);

            match &s.fields {
                Fields::Named(named) => {
                    let result = named
                        .named
                        .iter()
                        .map(|field| generator.generate(field))
                        .collect::<Vec<Result<_, _>>>();

                    if result.iter().any(Result::is_err) {
                        return Err(Error::multiple(result.into_iter().filter_map(Result::err)));
                    } else {
                        functions.extend(
                            result
                                .into_iter()
                                .filter_map(Result::ok)
                                .collect::<Vec<_>>(),
                        );
                    }
                }
                // A TupleStruct has no field names.
                Fields::Unnamed(_) => {
                    errors
                        .push(Error::custom("tuple structs are not supported.").with_span(&input));
                }
                // A Unit has no fields.
                Fields::Unit => {
                    errors.push(Error::custom("unit structs are not supported.").with_span(&input));
                }
            }
        }
        Data::Enum(_) => {
            errors.push(Error::custom("enum are not supported.").with_span(&input));
        }
        Data::Union(_) => {
            errors.push(Error::custom("union structs are not supported.").with_span(&input));
        }
    }

    if !errors.is_empty() {
        return Err(Error::multiple(errors));
    }

    let (impl_generics, ty_generics, where_clause) = options.generics.split_for_impl();

    if (options.attributes.try_into || options.attributes.into)
        && quote!(#impl_generics #ty_generics #where_clause)
            .to_string()
            .contains("VALUE")
    {
        return Err(
            Error::custom("a generic called `VALUE` is not supported, please rename it.")
                .with_span(&options.generics),
        );
    }

    Ok(quote! {
        #[allow(dead_code)]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#functions)*
        }
    })
}

struct Generator<'a> {
    options: &'a Options,
}

impl<'a> Generator<'a> {
    pub const fn from_options(options: &'a Options) -> Self { Self { options } }

    fn parse_template<T: AsRef<str>>(template: T, argument: &Ident) -> Result<Ident, Error> {
        let mut parts = template.as_ref().splitn(2, "{}");

        let prefix = parts.next().unwrap_or("");
        let suffix = parts
            .next()
            // this will return `None`, if the template doesn't contain `{}`, which should
            // never happen, because this is checked while parsing the format.
            .ok_or_else(|| Error::custom("missing `{}` in template."))?;

        Ok(format_ident!("{}{}{}", prefix, argument, suffix))
    }

    pub fn get(
        options: &Options,
        field_name: &Ident,
        field_type: &Type,
    ) -> Result<TokenStream, Error> {
        // apply the rename template, if there is none, use the default:
        // -> field: usize
        // -> with template `prefix_{}_suffix` -> prefix_field_suffix
        // -> without template -> `field`
        let function_name = {
            if options.attributes.rename {
                Self::parse_template(&options.rename.get_format, field_name).unwrap()
            } else {
                field_name.clone()
            }
        };

        let mut attributes: Vec<Attribute> = options.attrs.clone();
        let arguments = vec![quote![&self]];
        let visibility = options.visibility();
        let mut assertions = vec![];

        // add attributes to the function
        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }

        if options.attributes.must_use {
            attributes.push(Attribute::from_token_stream(quote!(#[must_use])).unwrap());
        }

        // change body, depending on the type and config:
        let (return_type, body) = {
            if options.attributes.primitive_copy && field_type.is_primitive_copy()
                || options.attributes.copy
            {
                (quote![#field_type], quote![self.#field_name])
            } else if options.attributes.option_as_ref && field_type.is_option() {
                // The getter will have the following signature
                // fn field(&self) -> Option<&String>;
                // instead of:
                // fn field(&self) -> &Option<String>;
                (
                    field_type.to_as_ref().unwrap(),
                    quote![self.#field_name.as_ref()],
                )
            } else {
                (quote![&#field_type], quote![&self.#field_name])
            }
        };

        // if the copy field has been enabled an assertion is needed, that ensures, that
        // the type implements `Copy`.
        if options.attributes.copy
            || (options.attributes.primitive_copy
                && field_type.is_primitive_copy()
                // the assertion does not work with lifetimes :(
                && !field_type.is_reference())
        {
            // this will expand to for example
            // struct _AssertCopy where usize: Copy;
            //
            // it acts as a check for wether a type implements Copy or not.
            assertions.push(quote_spanned! {
                field_type.span() =>
                struct _AssertCopy where #field_type: Copy;
            });
        }

        let const_fn = {
            if options.attributes.const_fn {
                quote![const]
            } else {
                quote![]
            }
        };

        Ok(quote! {
            #(#attributes)*
            #visibility #const_fn fn #function_name(#(#arguments),*) -> #return_type {
                #(#assertions)*
                #body
            }
        })
    }

    pub fn set(
        options: &Options,
        field_name: &Ident,
        field_type: &Type,
    ) -> Result<TokenStream, Error> {
        // apply the rename template, if there is none, use the default:
        // -> field: usize
        // -> with template `prefix_{}_suffix` -> prefix_field_suffix
        // -> without template -> `set_field`
        let function_name = {
            if options.attributes.rename {
                Self::parse_template(&options.rename.set_format, field_name).unwrap()
            } else {
                format_ident!("set_{}", field_name)
            }
        };

        let mut generics = vec![];
        let mut arguments = vec![quote![&mut self]];
        let return_type = quote![&mut Self];
        let visibility = options.visibility();

        let body = {
            if options.attributes.into {
                arguments.push(quote![value: VALUE]);
                generics.push(quote![VALUE: ::core::convert::Into<#field_type>]);
                quote![
                    self.#field_name = value.into();
                    self
                ]
            } else {
                arguments.push(quote![value: #field_type]);
                quote![
                    self.#field_name = value;
                    self
                ]
            }
        };

        // Attributes like `#[allow(clippy::use_self)]`
        let mut attributes: Vec<Attribute> = options.attrs.clone();

        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }

        // TODO: allow const for setters
        // Blocked by:  - rust-lang/rust#57349
        //              - rust-lang/rfcs#2632

        Ok(quote! {
            #(#attributes)*
            #visibility fn #function_name <#(#generics),*> ( #(#arguments),* ) -> #return_type {
                #body
            }
        })
    }

    pub fn try_set(
        options: &Options,
        field_name: &Ident,
        field_type: &Type,
    ) -> Result<TokenStream, Error> {
        let function_name = {
            if options.attributes.rename {
                Self::parse_template(&options.rename.try_set_format, field_name).unwrap()
            } else {
                format_ident!("try_{}", field_name)
            }
        };

        let mut attributes: Vec<Attribute> = options.attrs.clone();

        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }
        let visibility = options.visibility();

        Ok(quote! {
            #(#attributes)*
            #visibility fn #function_name<VALUE>(
                &mut self,
                value: VALUE
            ) -> Result<&mut Self, VALUE::Error>
            where
                VALUE: ::core::convert::TryInto<#field_type>
            {
                self.#field_name = value.try_into()?;
                Ok(self)
            }
        })
    }

    pub fn get_mut(
        options: &Options,
        field_name: &Ident,
        field_type: &Type,
    ) -> Result<TokenStream, Error> {
        let function_name = {
            if options.attributes.rename {
                Self::parse_template(&options.rename.get_mut_format, field_name).unwrap()
            } else {
                format_ident!("{}_mut", field_name)
            }
        };

        let mut attributes: Vec<Attribute> = options.attrs.clone();

        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }

        if options.attributes.must_use {
            attributes.push(Attribute::from_token_stream(quote!(#[must_use])).unwrap());
        }

        let visibility = options.visibility();

        Ok(quote! {
            #(#attributes)*
            #visibility fn #function_name(&mut self) -> &mut #field_type {
                &mut self.#field_name
            }
        })
    }

    pub fn collection_magic(
        options: &Options,
        field_name: &Ident,
        field_type: &Type,
    ) -> Result<TokenStream, Error> {
        let visibility = options.visibility();
        let mut attributes = options.attrs.clone();

        let type_name = Ident::new(&field_type.path().unwrap().to_string(), field_type.span());

        let mut function_name = None;
        let mut body = quote![];
        let mut arguments = vec![quote!(&mut self)];

        for (index, value) in field_type.arguments().unwrap().iter().enumerate() {
            let ident = format_ident!("value_{}", index);
            arguments.push(quote!(#ident: #value));
        }

        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }

        if options.attributes.must_use {
            attributes.push(Attribute::from_token_stream(quote!(#[must_use])).unwrap());
        }

        if field_type.is_ident("Vec") {
            function_name = Some(format_ident!("push_{}", field_name));
            body = quote_spanned! {
                field_type.span() =>
                struct __AssertVec(::std::vec::Vec<()>);
                __AssertVec(#type_name::new());
                self.#field_name.push(value_0);
                self
            };
        } else if field_type.is_ident("BTreeMap")
            || field_type.is_ident("BTreeSet")
            || field_type.is_ident("HashMap")
            || field_type.is_ident("HashSet")
        {
            let insert_args = (0..arguments.len() - 1).map(|i| format_ident!("value_{}", i));
            let assert_args = (0..arguments.len() - 1).map(|_| quote![()]);

            function_name = Some(format_ident!("insert_{}", field_name));
            body = quote_spanned! {
                field_type.span() =>
                struct __AssertCollection(::std::collections::#type_name<#(#assert_args),*>);
                __AssertCollection(#type_name::new());
                self.#field_name.insert(#(#insert_args),*);
                self
            };
        }

        if let Some(function_name) = function_name {
            Ok(quote! {
                #(#attributes)*
                #visibility fn #function_name(#(#arguments),*) -> &mut Self {
                    #body
                }
            })
        } else {
            Ok(quote!())
        }
    }

    /// This function generates the Functions for a [`Field`], based on the
    /// [`Options`] and the [`Field`] itself.
    pub fn generate(&self, field: &Field) -> Result<TokenStream, Error> {
        let options = self.options.with_attrs(&field.attrs)?;

        let field_name = {
            if let Some(ident) = field.ident.as_ref() {
                ident
            } else {
                // This shouldn't be reached, because expand::derive ensures, that all fields
                // have a name.
                unreachable!("unnamed field guard failed");
            }
        };

        let mut result = quote![];

        if (options.attributes.ignore_phantomdata && field.ty.is_ident("PhantomData"))
            || options.attributes.skip
            || (options.attributes.ignore_underscore && {
                field
                    .ty
                    .path()
                    .map_or(false, |p| p.to_string().starts_with('_'))
            })
            || {
                // empty tuple
                if let syn::Type::Tuple(s) = &field.ty {
                    s.elems.is_empty()
                } else {
                    false
                }
            }
            || {
                if let syn::Type::Never(_) = &field.ty {
                    true
                } else {
                    false
                }
            }
        {
            return Ok(quote![]);
        }

        if options.attributes.get {
            let function = Self::get(&options, field_name, &field.ty)?;
            result = quote! {
                #result
                #function
            };
        }

        if options.attributes.set {
            let function = Self::set(&options, field_name, &field.ty)?;
            result = quote! {
                #result
                #function
            };
        }

        if options.attributes.try_into {
            let function = Self::try_set(&options, field_name, &field.ty)?;
            result = quote! {
                #result
                #function
            };
        }

        if options.attributes.get_mut {
            let function = Self::get_mut(&options, field_name, &field.ty)?;
            result = quote! {
                #result
                #function
            };
        }

        if options.attributes.collection_magic {
            let function = Self::collection_magic(&options, field_name, &field.ty)?;
            result = quote! {
                #result
                #function
            };
        }

        Ok(result)
    }
}
