#![doc(test(attr(deny(unused_mut))))]
//! # shorthand
//! The Cambridge Dictionary defines
//! [`shorthand`](https://dictionary.cambridge.org/de/worterbuch/englisch/shorthand) as
//! `a system of fast writing`
//! and that is exactly what this library is for; to remove the annoying boilerplate code,
//! that comes with writing your own library.
//!
//! # What does this library do?
//!
//! It makes coding in rust a lot more convenient, by deriving `getters` and `setters`
//! for the fields of a struct.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! pub struct Example {
//!     number: usize,
//!     data: String,
//! }
//!
//! let mut example = Example::default();
//!
//! assert_eq!(example.number(), 0);
//! example.set_number(1);
//! assert_eq!(example.number(), 1);
//!
//! assert_eq!(example.data(), &"".to_string());
//! example.set_data("hi".to_string());
//! assert_eq!(example.data(), &"hi".to_string());
//! ```
//!
//! Otherwise, you would have to write the this by hand
//!
//! ```
//! # pub struct Example {
//! #     number: usize,
//! #     data: String,
//! # }
//! #[allow(dead_code)]
//! impl Example {
//!     #[inline(always)]
//!     pub fn number(&self) -> usize {
//!         self.number
//!     }
//!     #[inline(always)]
//!     pub fn set_number(&mut self, value: usize) -> &mut Self {
//!         self.number = value;
//!         self
//!     }
//!     #[inline(always)]
//!     pub fn data(&self) -> &String {
//!         &self.data
//!     }
//!     #[inline(always)]
//!     pub fn set_data(&mut self, value: String) -> &mut Self {
//!         self.data = value;
//!         self
//!     }
//! }
//! ```
//!
//! # How do I get started?
//!
//! Simply add this library under `[dependencies]` to your `Cargo.toml`
//! ```toml
//! [dependencies]
//! shorthand = "0.1.0"
//! ```
//!
//! You can then derive `ShortHand` for any struct
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand)]
//! struct Example {
//!     field: usize,
//! }
//! ```
//!
//! # Customization
//!
//! The derive macro can be heavily customized with the `#[shorthand]` attribute.
//! It has the following main attributes:
//! * [`visibility`](#visibility)
//! * [`rename`](#rename)
//! * [`enable`](#enable)
//! * [`disable`](#disable)
//!
//! ## `disable`
//!
//! This attribute allows you to disable certain attributes. For example by default the attribute
//! [`primitive_copy`](#primitive_copy) is enabled
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(disable(primitive_copy))]
//!     field: usize,
//!     other: usize,
//! }
//!
//! let example = Example::default();
//!
//! assert_eq!(example.field(), &0); // returns a reference, instead of copying the value
//! assert_eq!(example.other(), 0);
//! ```
//!
//! You can find a list with all attributes, that can be disabled [here](#attributes).
//!
//! ## `enable`
//!
//! This attribute allows you to enable certain attributes. For example by default the attribute
//! [`into`](#into) is disabled.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(enable(into))]
//!     field: String,
//!     other: String,
//! }
//!
//! let mut example = Example::default();
//!
//! example.set_field("field"); // accepts any type, that implements Into<String> or From<String>
//! example.set_other("other".to_string());
//!
//! assert_eq!(example.field(), &"field".to_string());
//! assert_eq!(example.other(), &"other".to_string());
//! ```
//!
//! You can find a list with all attributes, that can be enabled [here](#attributes).
//! Attributes, that are [enabled by default](#enabled-by-default).
//!
//! ## `visibility`
//!
//! This attribute allows you to change the visibility of the derived function.
//! Anything from
//! [here](https://doc.rust-lang.org/reference/visibility-and-privacy.html#visibility-and-privacy)
//! is valid.
//! You can either apply this as a local or as a global attribute.
//! The default visibility is `pub`.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! #[shorthand(visibility(pub(crate)))]
//! struct Example {
//!     field: usize,
//!     #[shorthand(visibility(pub))]
//!     data: String,
//! }
//! ```
//!
//! ## `rename`
//!
//! This attribute allows you to rename the derived function, with a pattern.
//! You can either apply this as a local or as a global attribute.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! #[shorthand(rename("prefix_{}_suffix"))]
//! struct Example {
//!     field: usize,
//!     #[shorthand(rename(format = "example_{}"))]
//!     data: String,
//! }
//!
//! let mut example = Example::default();
//! example.set_prefix_field_suffix(1);
//! example.set_example_data("Hello".to_string());
//!
//! assert_eq!(example.prefix_field_suffix(), 1);
//! assert_eq!(example.example_data(), &"Hello".to_string());
//! ```
//!
//! This attribute also supports changing the getter and setter individually:
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! #[shorthand(rename(format = "prefix_{}_suffix"))]
//! struct Example {
//!     #[shorthand(rename(get = "get_{}", set = "set_{}_a"))]
//!     field: usize,
//!     #[shorthand(rename(get = "get_{}"))] // this will not change the setter
//!     data: String,
//! }
//!
//! let mut example = Example::default();
//! example.set_field_a(1);
//! example.set_data("Hello".to_string());
//!
//! assert_eq!(example.get_field(), 1);
//! assert_eq!(example.get_data(), &"Hello".to_string());
//! ```
//!
//! In the case, that you have a rename attribute on the entire struct, but you do not want to
//! apply it for one specific field, you can disable it.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! #[shorthand(rename("prefix_{}_suffix"))]
//! struct Example {
//!     #[shorthand(disable(rename))]
//!     field: usize,
//!     data: String,
//! }
//!
//! let mut example = Example::default();
//! example.set_field(1);
//! example.set_prefix_data_suffix("Hello".to_string());
//!
//! assert_eq!(example.field(), 1);
//! assert_eq!(example.prefix_data_suffix(), &"Hello".to_string());
//! ```
//!
//! ## Attributes
//!
//! There are multiple attributes, that can be [`enable`]d or [`disable`]d:
//! - [`option_as_ref`](#option_as_ref)
//! - [`const_fn`](#const_fn)
//! - [`primitive_copy`](#primitive_copy)
//! - [`inline`](#inline)
//! - [`must_use`](#must_use)
//! - [`copy`](#copy)
//! - [`get`](#get)
//! - [`set`](#set)
//! - [`ignore_phantomdata`](#ignore_phantomdata)
//! - [`skip`](#skip)
//! - [`rename`](#rename)
//! - [`into`](#into)
//! - [`forward_attributes`](#forward_attributes)
//! - [`forward_everything`](#forward_everything)
//!
//! ### Enabled by default
//!
//! The following attributes are [`enable`]d by default
//! - [`option_as_ref`](#option_as_ref)
//! - [`primitive_copy`](#primitive_copy)
//! - [`inline`](#inline)
//! - [`get`](#get)
//! - [`set`](#set)
//! - [`ignore_phantomdata`](#ignore_phantomdata)
//! - [`forward_attributes`](#forward_attributes)
//!
//! [`enable`]: #enable
//! [`disable`]: #disable
//!
//! ### `option_as_ref`
//! This attribute makes the getter return `Option<&T>` instead of `&Option<T>`.
//! This feature is enabled by default and recommended, because most of the functions for
//! [`Option`] consume the type.
//!
//! You can find a discussion, about wether or not you should use it
//! [here](https://users.rust-lang.org/t/api-design-option-t-vs-option-t/34139).
//!
//! ### `const_fn`
//! There is a new feature coming to rust called constant functions.
//! Functions, that are marked with `const` can be executed by the compiler at compile time,
//! if the value is known.
//!
//! ```
//! const fn add(value: usize, other: usize) -> usize {
//!     value + other
//! }
//!
//! let three = add(1, 2);
//! # assert_eq!(three, 3);
//! ```
//!
//! will be optimized to
//!
//! ```
//! let three = 3;
//! ```
//!
//! Another benefit is, that you can also save the result in a `const` variable.
//!
//! ```
//! const fn add(value: usize, other: usize) -> usize {
//!     value + other
//! }
//!
//! const THREE: usize = add(1, 2);
//! ```
//!
//! This feature is currently work in progress see
//! [rust-lang/rust#57563](https://github.com/rust-lang/rust/issues/57563)
//!
//! You can read more about it
//! [here](https://doc.rust-lang.org/unstable-book/language-features/const-fn.html)
//! or in the [RFC](https://github.com/rust-lang/rfcs/blob/master/text/0911-const-fn.md).
//!
//! By enabling this feature the derived functions will have the `const` attribute added.
//!
//! Please note, that not everything is currently supported and therefore some attributes will
//! ignore this attribute and not add `const` to the function.
//!
//! ### `primitive_copy`
//!
//! This attribute will cause get functions to return a copy of the type, instead of a reference.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! #[shorthand(enable(primitive_copy))]
//! struct Example {
//!     field: usize,
//!     #[shorthand(disable(primitive_copy))]
//!     other: usize,
//! }
//!
//! let example = Example::default();
//!
//! assert_eq!(example.field(), 0);
//! assert_eq!(example.other(), &0);
//! ```
//!
//! This attribute is enabled by default.
//!
//! Please note, that this does only work with primitive types from the standard library,
//! other types have to be marked with [`copy`](#copy).
//!
//! ### `copy`
//!
//! There is no way for a [`proc_macro`] to know wether or not a type implements [`Copy`],
//! so fields, where the getter should return a copy, instead of a reference have to be marked
//! with `#[shorthand(copy)]`.
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(Default, Copy, Clone, PartialEq, Debug)]
//! struct Number(pub usize);
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(enable(copy))]
//!     field: Number,
//!     other: Number,
//! }
//!
//! let example = Example::default();
//!
//! assert_eq!(example.field(), Number(0));
//! assert_eq!(example.other(), &Number(0));
//! ```
//!
//! ### `inline`
//!
//! This attribute adds `#[inline(always)]` to the derived function.
//!
//! ```
//! #[inline(always)]
//! fn add(a: usize, b: usize) -> usize {
//!     a + b
//! }
//!
//! let three = add(1, 2);
//! ```
//!
//! will be optimized to
//!
//! ```
//! let three = 1 + 2;
//! ```
//!
//! You can read more about the inline attribute
//! [here](https://doc.rust-lang.org/reference/attributes/codegen.html#the-inline-attribute).
//!
//! A discussion, about wether or not you should use it:
//! <https://internals.rust-lang.org/t/when-should-i-use-inline/598>
//!
//! This attribute is enabled by default.
//!
//! ### `must_use`
//!
//! This attribute will mark functions with `#[must_use]`,
//! which means, that their results have to be used, otherwise you get a warning
//!
//! ```
//! #[must_use]
//! fn hello() -> &'static str {
//!     "hello"
//! }
//!
//! hello();
//! ```
//!
//! ```text
//! warning: unused return value of `hello` that must be used
//!  --> src/main.rs:6:5
//!   |
//! 6 |     hello();
//!   |     ^^^^^^^^
//!   |
//!   = note: `#[warn(unused_must_use)]` on by default
//! ```
//!
//! This is disabled by default and can only be enabled for getter and mutable getter.
//! If you really need this attributes on a setter you can just mark the field with
//! `#[must_use]` and shorthand will automatically forward the attribute
//! (see [here](#forward_attributes)).
//!
//! ### `get`
//!
//! This attribute derives a function, to get the value of a field
//! (sometimes referred to as `getter`).
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(enable(get))]
//!     field: usize,
//! }
//!
//! let example = Example::default();
//!
//! assert_eq!(example.field(), 0);
//! ```
//!
//! This attribute is enabled by default.
//!
//! ### `set`
//!
//! This attribute derives a function, to set the value of a field
//! (sometimes referred to as `setter`).
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(enable(get, set))]
//!     field: usize,
//! }
//!
//! let mut example = Example::default();
//!
//! example.set_field(1);
//!
//! assert_eq!(example.field(), 1);
//! ```
//!
//! This attribute is enabled by default.
//!
//! ### `ignore_phantomdata`
//!
//! Like the name implies, this will tell the [`proc_macro`], to ignore [`PhantomData`] and
//! to not generate functions for it.
//!
//! ```compile_fail
//! use shorthand::ShortHand;
//! use core::marker::PhantomData;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     field: PhantomData<usize>,
//! }
//!
//! let example = Example::default();
//! example.field(); // this will cause a compiler error, because the function does not exist!
//! ```
//!
//! This feature is enabled by default.
//!
//! [`PhantomData`]: core::marker::PhantomData
//!
//! ### `skip`
//!
//! Like the name implies, this will tell the [`proc_macro`] to not generate functions for this
//! field (skipping it).
//!
//! ```compile_fail
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(enable(skip))]
//!     field: usize,
//! }
//!
//! let example = Example::default();
//! example.field(); // this will cause a compiler error, because the function does not exist!
//! ```
//!
//! ### `into`
//!
//! The `into` attribute adds `VALUE: Into<field_type>` as a trait bound for setters ([`Into`]).
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand, Default)]
//! struct Example {
//!     #[shorthand(enable(into))]
//!     field: String,
//!     other: String,
//! }
//!
//! let mut example = Example::default();
//!
//! example.set_field("field"); // accepts any type, that implements Into<String> or From<String>
//! example.set_other("other".to_string());
//!
//! assert_eq!(example.field(), &"field".to_string());
//! assert_eq!(example.other(), &"other".to_string());
//! ```
//!
//! This struct uses `VALUE` as a generic, so you should *NOT* use that on your struct
//!
//! ```
//! struct DoNotDoThis<VALUE> {
//!     value: VALUE,
//! }
//! ```
//!
//! This attribute is not enabled by default.
//!
//! ### `forward_attributes`
//!
//! The `forward_attributes` attribute, allows you to control how attributes are forwarded.
//! By default this is enabled and the [`proc_macro`] will forward the following attributes of the
//! field to the function:
//!
//! - [`doc`](https://doc.rust-lang.org/rustdoc/the-doc-attribute.html)
//! - [`cfg`](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-attribute)
//! - [`cfg_attr`](https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute)
//! - [`allow`], [`warn`], [`deny`], [`forbid`]
//! - [`deprecated`]
//! - [`must_use`]
//! - [`inline`]
//! - [`cold`]
//! - [`target_feature`]
//!
//! [`allow`]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes
//! [`warn`]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes
//! [`deny`]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes
//! [`forbid`]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes
//! [`deprecated`]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-deprecated-attribute
//! [`must_use`]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute
//! [`inline`]: https://doc.rust-lang.org/reference/attributes/codegen.html#the-inline-attribute
//! [`cold`]: https://doc.rust-lang.org/reference/attributes/codegen.html#the-cold-attribute
//! [`target_feature`]: https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute
//!
//! If you want to forward everything, you can use [`forward_everything`](#forward_everything).
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand)]
//! struct Example {
//!     #[shorthand(enable(forward_attributes))]
//!     #[must_use]
//!     hello: &'static str,
//! }
//! ```
//!
//! The `hello` function would now have a `#[must_use]` attribute:
//! ```
//! #[must_use]
//! fn hello() -> &'static str {
//!     "hello"
//! }
//! ```
//!
//! This attribute can be applied multiple times on the same field, which allows for controlled
//! forwarding
//!
//! ```
//! use shorthand::ShortHand;
//!
//! #[derive(ShortHand)]
//! struct Example {
//!     #[shorthand(enable(forward_attributes))]
//!     #[must_use]
//!     #[shorthand(disable(forward_attributes))]
//!     #[allow(dead_code)]
//!     hello: &'static str,
//! }
//! ```
//!
//! In this example only `#[must_use]` is forwarded and `#[allow(dead_code)]` is not.
//!
//! ### `forward_everything`
//!
//! The `forward_everything` attribute will like the name implies instruct the [`proc_macro`] to
//! forward all attributes, that a field has.
//! This attribute works like [`forward_attributes`](#forward_attributes).
//!
//! This attribute is not enabled by default.
//!
//!
//! # Feature Requests and Bug Reports
//!
//! Feel free to ask questions or report bugs [here](https://www.github.com/luro02/shorthand).
//! There are no stupid questions.
//!
//! This library should be as convenient as possible, so please do not hesitate to
//! request a feature.
//!
//! # Planned Features
//! - function documentation `#[shorthand(doc(file = "", function = ""))]`
//! - mut getter
//! - `no_std`
//!
//! # Reference
//!
//! This library has been inspired by the following crates
//! - [`getset`] (just the issue tracker and which features were requested)
//! - [`thiserror`]
//! - [`derive-builder`]
//! - [`proc-macro-workshop`]
//!
//! [`getset`]: https://github.com/Hoverbear/getset
//! [`thiserror`]: https://github.com/dtolnay/thiserror
//! [`derive-builder`]: https://github.com/colin-kiegel/rust-derive-builder
//! [`proc-macro-workshop`]: https://github.com/dtolnay/proc-macro-workshop
//! [`proc_macro`]: https://doc.rust-lang.org/reference/procedural-macros.html
#![deny(clippy::use_self, unconditional_recursion)]
#![warn(clippy::pedantic, clippy::nursery)]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    //missing_docs
)]
#![allow(clippy::must_use_candidate, clippy::default_trait_access)]
extern crate proc_macro;

mod attributes;
mod error;
mod expand;
mod options;
mod utils;
mod visibility;

pub(crate) use error::Error;
pub(crate) type Result<T> = std::result::Result<T, Error>;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

// TODO: move parts from above inside this
/// Please see [here](../shorthand/index.html) for usage of the macro.
#[proc_macro_derive(ShortHand, attributes(shorthand))]
pub fn shorthand(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(error::Error::into_token_stream)
        .into()
}
