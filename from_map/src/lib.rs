#![warn(clippy::pedantic, clippy::nursery)]
//! This crate is based on
//! <https://cprimozic.net/blog/writing-a-hashmap-to-struct-procedural-macro-in-rust/>
//!
//! There was sadly no crate available, so I had to make my own :(
//!
//! I made some improvements to the code and ported it to a newer version of
//! `syn`. For example the [`FromMap`] trait doesn't need a type parameter
//! and the [`HashMap`] should contain a static str (field names should be known
//! at compile time).
pub use hashmap_derive::FromMap;
use std::collections::HashMap;

pub trait FromMap: Default {
    type Value;

    #[must_use]
    fn from_map(input: &HashMap<&'static str, Self::Value>) -> Self {
        let mut result = Self::default();
        FromMap::with_map(&mut result, input);
        result
    }

    fn with_map(&mut self, input: &HashMap<&'static str, Self::Value>);

    fn as_map(&self) -> HashMap<&'static str, Self::Value> { HashMap::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, FromMap, Debug, PartialEq)]
    struct Attributes {
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
        pub rename: bool,
    }

    #[test]
    fn it_works() {
        let mut data = HashMap::new();
        data.insert("option_as_ref", true);
        data.insert("const_fn", true);
        data.insert("primitive_copy", true);
        data.insert("inline", true);
        data.insert("must_use", true);
        data.insert("copy", true);
        data.insert("get", true);
        data.insert("set", true);
        data.insert("ignore_phantomdata", true);
        data.insert("skip", true);
        data.insert("rename", true);

        let result = Attributes::from_map(&data);

        assert_eq!(
            result,
            Attributes {
                option_as_ref: true,
                const_fn: true,
                primitive_copy: true,
                inline: true,
                must_use: true,
                copy: true,
                get: true,
                set: true,
                ignore_phantomdata: true,
                skip: true,
                rename: true,
            }
        );
    }
}
