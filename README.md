# cargo-snippet

[![crates.io](https://img.shields.io/crates/v/cargo-snippet.svg)](https://crates.io/crates/cargo-snippet)
[![Build Status](https://travis-ci.org/hatoo/cargo-snippet.svg?branch=master)](https://travis-ci.org/hatoo/cargo-snippet)
[![dependency status](https://deps.rs/repo/github/hatoo/cargo-snippet/status.svg)](https://deps.rs/repo/github/hatoo/cargo-snippet)

A snippet extractor for competitive programmers.

This allows you to manage your code snippets with tests and benchmarks available !!

## Installing

You need to install `rustfmt` to run `cargo-snippet`.

```bash
$ rustup component add rustfmt
```

Install `cargo-snippet`

```bash
$ cargo install cargo-snippet --features="binaries"
```

## Usage

Create a project for snippet.

```
$ cargo new --lib mysnippet
```

Add dependencies to Cargo.toml.

```toml
[dependencies]
cargo-snippet = "0.6"
```

Note: `cargo-snippet` on dependencies is needed just for register `#[snippet]` attribute to prevent the error from the compiler.
All logics that extract snippet is in the binary package which is installed by `Installing` section.

Then write some snippet codes and tests.

```rust
use cargo_snippet::snippet;

// Annotate snippet name
#[snippet("mymath")]
#[snippet("gcd")]
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// Also works
#[snippet(name = "mymath")]
// Equivalent to #[snippet("lcm")]
#[snippet]
fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
}

#[snippet]
// Include snippet
#[snippet(include = "gcd")]
fn gcd_list(list: &[u64]) -> u64 {
    list.iter().fold(list[0], |a, &b| gcd(a, b))
}

// You can set prefix string.
// Note: All codes will be formatted by rustfmt on output
#[snippet(prefix = "use std::io::{self,Read};")]
#[snippet(prefix = "use std::str::FromStr;")]
fn foo() {}

// By default, doc comments associated with items will be output with the snippet.
#[snippet]
/// This is a document!
fn documented() {
    //! Inner document also works.
}

// If you want doc comment to be hidden, append `doc_hidden` keyword.
#[snippet(doc_hidden, prefix = "use std::collections::HashMap;")]
/// This is a doc comment for `bar`.
/// Since `doc_hidden` is specified, it won't be present in the snippet.
fn bar() {
    //! And this is also a doc comment for `bar`, which will be removed.
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(57, 3), 3);
}

#[test]
fn test_lcm() {
    assert_eq!(lcm(3, 19), 57);
}
```

You can test as always:

```
$ cargo test
```

Extract snippet !

```
$ cargo snippet
snippet foo
    use std::io::{self, Read};
    use std::str::FromStr;
    fn foo() {}

snippet documented
    /// This is a document!
    fn documented() {
        //! Inner document also works.
    }

snippet bar
    use std::collections::HashMap;
    fn bar() {}

snippet gcd
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

snippet gcd_list
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    fn gcd_list(list: &[u64]) -> u64 {
        list.iter().fold(list[0], |a, &b| gcd(a, b))
    }

snippet lcm
    fn lcm(a: u64, b: u64) -> u64 {
        a / gcd(a, b) * b
    }

snippet mymath
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    fn lcm(a: u64, b: u64) -> u64 {
        a / gcd(a, b) * b
    }

```

## Example

My snippets [here](https://github.com/hatoo/competitive-rust-snippets.git).

## Supported output format

* Neosnippet
* VScode
* Ultisnips

You can specify output format via `-t` option.
See `cargo snippet -h`.
