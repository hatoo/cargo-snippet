# cargo-snippet

[![crates.io](https://img.shields.io/crates/v/cargo-snippet.svg)](https://crates.io/crates/cargo-snippet)
[![Build Status](https://travis-ci.org/hatoo/cargo-snippet.svg?branch=master)](https://travis-ci.org/hatoo/cargo-snippet)

A snippet extractor for competitive programmers.

You can manage code snippet with test and bench !!

## Installing

You need nightly rust.

```
$ cargo install cargo-snippet --features="binaries"
```

## Usage

Create a project for snippet.

```
$ cargo new mysnippet
```

Add dependencies to Cargo.toml.

```toml
[dependencies]
cargo-snippet = "0.1"
```

Add this to src/lib.rs.

```rust
#![feature(plugin)]
#![plugin(cargo_snippet)]
```

Write some snippet codes and tests.

```rust
#![feature(plugin)]
#![plugin(cargo_snippet)]

// Annotate snippet name
#[snippet = "mymath"]
#[snippet = "gcd"]
#[allow(dead_code)]
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// Also works
#[snippet(name = "mymath")]
#[allow(dead_code)]
fn lcm(a: u64, b: u64) -> u64 {
    a / gcd(a, b) * b
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

You can test.

```
$ cargo test
```

Extract snippet !

```
$ cargo snippet
snippet gcd
    #[allow(dead_code)]
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }

snippet mymath
    #[allow(dead_code)]
    fn gcd(a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
    #[allow(dead_code)]
    fn lcm(a: u64, b: u64) -> u64 {
        a / gcd(a, b) * b
    }
```

## Example

My snippets [here](https://github.com/hatoo/competitive-rust-snippets.git).

## Supported output format

* Neosnippet
* VScode

You can specify output format via `-t` option.
See `cargo snippet -h`.