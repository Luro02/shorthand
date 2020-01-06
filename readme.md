shorthand
===
[![Crates.io: hex](https://img.shields.io/crates/v/shorthand.svg)](https://crates.io/crates/shorthand)
[![Documentation](https://docs.rs/shorthand/badge.svg)](https://docs.rs/shorthand)
[![Build Status](https://travis-ci.org/Luro02/shorthand.svg?branch=master)](https://travis-ci.org/Luro02/shorthand)

[`shorthand`](https://dictionary.cambridge.org/de/worterbuch/englisch/shorthand) is defined as
`a system of fast writing`
and that is exactly what this library is for; to remove the annoying
boilerplate code, that comes with writing your own library.

## What does this library do?

It makes coding in rust a lot more convenient, by deriving `getters` and
`setters` for the fields of a struct.

```rust
use shorthand::ShortHand;

#[derive(ShortHand, Default)]
pub struct Example {
    number: usize,
    data: String,
}

let mut example = Example::default();

assert_eq!(example.number(), 0);
example.set_number(1);
assert_eq!(example.number(), 1);

assert_eq!(example.data(), &"".to_string());
example.set_data("hi".to_string());
assert_eq!(example.data(), &"hi".to_string());
```

Otherwise, you would have to write the this by hand

```rust
pub struct Example {
    number: usize,
    data: String,
}

#[allow(dead_code)]
impl Example {
    #[inline(always)]
    pub fn number(&self) -> usize { self.number }

    #[inline(always)]
    pub fn set_number(&mut self, value: usize) -> &mut Self {
        self.number = value;
        self
    }

    #[inline(always)]
    pub fn data(&self) -> &String { &self.data }

    #[inline(always)]
    pub fn set_data(&mut self, value: String) -> &mut Self {
        self.data = value;
        self
    }
}
```

## How do I get started?

Simply add this library under `[dependencies]` to your `Cargo.toml`
```toml
[dependencies]
shorthand = "0.1.0"
```

You can then derive `ShortHand` for any struct

```rust
use shorthand::ShortHand;

#[derive(ShortHand)]
struct Example {
    field: usize,
}
```

You can find the [documentation here](https://docs.rs/shorthand).

## Feature Requests and Bug Reports

Feel free to ask questions or report bugs [here](https://www.github.com/luro02/shorthand).
There are no stupid questions.

This library should be as convenient as possible, so please do not hesitate
to request a feature.

## Planned Features
- function documentation `#[shorthand(doc(file = "", function = ""))]`
- mut getter
- `no_std`

## Reference

This library has been inspired by the following crates
- [`getset`] (just the issue tracker and which features were requested)
- [`thiserror`]
- [`derive-builder`]
- [`proc-macro-workshop`]

[`getset`]: https://github.com/Hoverbear/getset
[`thiserror`]: https://github.com/dtolnay/thiserror
[`derive-builder`]: https://github.com/colin-kiegel/rust-derive-builder
[`proc-macro-workshop`]: https://github.com/dtolnay/proc-macro-workshop
[`proc_macro`]: https://doc.rust-lang.org/reference/procedural-macros.html
