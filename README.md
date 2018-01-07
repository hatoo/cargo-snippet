# cargo-snippet

Snippet extractor for competitive programmers.

You can manage code snippet with test and bench !!

## Installing

You need nightly rust.

```
$ cargo install --git https://github.com/hatoo/cargo-snippet.git --features="binaries"
```

## Usage

Create a project for snippet.

```
$ cargo new mysnippet
```

Add dependencies to Cargo.toml.

```toml
[dependencies]
cargo-snippet = { git = "https://github.com/hatoo/cargo-snippet.git" }
```

Add this to src/lib.rs.

```rust
#![feature(plugin)]
#![plugin(cargo_snippet)]
```

write some snippet code and test.

```rust
#![feature(plugin)]
#![plugin(cargo_snippet)]

// Annotate snippet name
#[snippet = "gcd"]
#[allow(dead_code)]
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// Also OK
#[snippet(name = "lcm")]
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

snippet lcm
    #[allow(dead_code)]
    fn lcm(a: u64, b: u64) -> u64 {
        a / gcd(a, b) * b
    }
```

## Supported output format

Currently this only supports neosnippet format.