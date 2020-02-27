use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned as _;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, Generics, Ident, PredicateType, TraitBound,
    TraitBoundModifier, Type, TypeParamBound, WherePredicate,
};

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
        #[allow(clippy::all)]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#functions)*
        }
    })
}

fn generate_assertion(
    name: &TokenStream,
    field_type: &Type,
    generics: &Generics,
    bound: &TokenStream,
) -> TokenStream {
    let where_clause = {
        if let Some(where_clause) = &generics.where_clause {
            let mut result = (*where_clause).clone();

            result
                .predicates
                .push_value(WherePredicate::Type(PredicateType {
                    lifetimes: None,
                    bounded_ty: field_type.clone(),
                    colon_token: syn::parse2(quote!(:)).unwrap(),
                    bounds: {
                        let mut bounds = Punctuated::new();

                        bounds.push_value(TypeParamBound::Trait(TraitBound {
                            paren_token: None,
                            modifier: TraitBoundModifier::None,
                            lifetimes: None,
                            path: syn::parse2(quote!(#bound)).unwrap(),
                        }));

                        bounds
                    },
                }));

            quote!(#result)
        } else {
            quote!( where #field_type: #bound )
        }
    };

    let fields = {
        let mut result = vec![];
        let mut field_number: usize = 0;

        for lifetime_def in generics.lifetimes() {
            let lifetime = &lifetime_def.lifetime;
            let name = format_ident!("__field_{}", field_number);

            result.push(quote! {
                #name: ::std::marker::PhantomData<& #lifetime ()>
            });
            field_number += 1;
        }

        for param in generics.type_params() {
            let ident = &param.ident;
            let name = format_ident!("__field_{}", field_number);

            result.push(quote! {
                #name: ::std::marker::PhantomData<#ident>
            });
            field_number += 1;
        }

        result
    };

    let (_, ty_generics, _) = generics.split_for_impl();

    // this will expand to for example
    // struct _AssertCopy where usize: Copy;
    //
    // it acts as a check for wether a type implements Copy or not.
    quote_spanned! {
        field_type.span() =>
        struct #name #ty_generics #where_clause {
            #(#fields),*
        }
    }
}

struct Generator<'a> {
    options: &'a Options,
}

impl<'a> Generator<'a> {
    pub const fn from_options(options: &'a Options) -> Self { Self { options } }

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
                options.rename.format_get(field_name)?
            } else {
                field_name.clone()
            }
        };

        let mut attributes: Vec<Attribute> = options.attrs.clone();
        let arguments = vec![quote![&self]];
        let visibility = &options.visibility;
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
            } else if options.attributes.clone {
                assertions.push(generate_assertion(
                    &quote!(_AssertClone),
                    field_type,
                    &options.generics,
                    &quote!(::std::clone::Clone),
                ));

                (quote![#field_type], quote! { self.#field_name.clone() })
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
            assertions.push(generate_assertion(
                &quote!(_AssertCopy),
                field_type,
                &options.generics,
                &quote!(::std::marker::Copy),
            ));
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
                options.rename.format_set(field_name)?
            } else {
                format_ident!("set_{}", field_name)
            }
        };

        let mut generics = vec![];
        let mut arguments = vec![quote![&mut self]];
        let return_type = quote![&mut Self];
        let visibility = &options.visibility;

        let mut argument = quote! { value: #field_type };
        let mut assignment = quote! { self.#field_name = value; };

        if options.attributes.into {
            argument = quote! { value: VALUE };
            let mut bound = quote! { VALUE: ::std::convert::Into<#field_type> };

            // default assignment for into
            assignment = quote! {
                self.#field_name = value.into();
            };

            // For Option we might want to have
            //
            // fn set_field<T: Into<String>>(value: Option<T>);
            //
            // instead of
            //
            // fn set_field<T: Into<Option<String>>>(value: T);
            //
            if field_type.is_ident("Option") {
                // tries to get the `T` from Option<T>
                if let Some(arg) = field_type
                    .arguments()
                    .into_iter()
                    .find_map(|s| s.into_iter().last())
                {
                    bound = quote! { VALUE: ::std::convert::Into<#arg> };

                    if options.attributes.strip_option {
                        assignment = quote! {
                            self.#field_name = Some(value.into());
                        };
                    } else {
                        argument = quote! { value: ::std::option::Option<VALUE> };

                        assignment = quote! {
                            self.#field_name = value.map(|v| v.into());
                        };
                    }
                }
            }

            generics.push(bound);
        } else if field_type.is_ident("Option") && options.attributes.strip_option {
            if let Some(arg) = field_type
                .arguments()
                .into_iter()
                .find_map(|s| s.into_iter().last())
            {
                argument = quote! { value: #arg };

                assignment = quote! {
                    self.#field_name = Some(value);
                };
            }
        }

        arguments.push(argument);

        // Attributes like `#[allow(clippy::use_self)]`
        let mut attributes: Vec<Attribute> = options.attrs.clone();

        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }

        // TODO: allow const for setters
        // Blocked by:  - rust-lang/rust#57349
        //              - rust-lang/rfcs#2632

        let verify = &options.verify;

        Ok(quote! {
            #(#attributes)*
            #visibility fn #function_name <#(#generics),*> ( #(#arguments),* ) -> #return_type {
                #assignment
                #verify
                self
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
                options.rename.format_try_set(field_name)?
            } else {
                format_ident!("try_{}", field_name)
            }
        };

        let mut attributes: Vec<Attribute> = options.attrs.clone();
        let mut argument = quote! { value: VALUE };

        if options.attributes.inline {
            attributes.push(Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());
        }
        let visibility = &options.visibility;

        let mut body = quote! {
            self.#field_name = value.try_into()?;
        };

        let mut bound = quote! {
            VALUE: ::std::convert::TryInto<#field_type>
        };

        if field_type.is_ident("Option") {
            if let Some(arg) = field_type
                .arguments()
                .into_iter()
                .find_map(|s| s.into_iter().last())
            {
                if options.attributes.strip_option {
                    body = quote! {
                        self.#field_name = Some(value.try_into()?);
                    };
                } else {
                    argument = quote! { value: ::std::option::Option<VALUE> };
                    body = quote! {
                        self.#field_name = value.map(|v| v.try_into()).transpose()?;
                    };
                }

                bound = quote! {
                    VALUE: ::std::convert::TryInto<#arg>
                };
            }
        }

        let verify = &options.verify;

        Ok(quote! {
            #(#attributes)*
            #visibility fn #function_name<VALUE>(
                &mut self,
                #argument
            ) -> Result<&mut Self, VALUE::Error>
            where
                #bound
            {
                #body
                #verify
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
                options.rename.format_get_mut(field_name)?
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

        let visibility = &options.visibility;

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
        let visibility = &options.visibility;
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

        #[allow(clippy::collapsible_if)]
        {
            if options.attributes.collection_magic {
                if field.ty.is_ident("Vec")
                    || field.ty.is_ident("BTreeMap")
                    || field.ty.is_ident("BTreeSet")
                    || field.ty.is_ident("HashMap")
                    || field.ty.is_ident("HashSet")
                {
                    let function = Self::collection_magic(&options, field_name, &field.ty)?;
                    result = quote! {
                        #result
                        #function
                    };
                }
            }
        }
        Ok(result)
    }
}
