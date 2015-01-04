# rustc-serialize

Serialization and deserialization support provided by the compiler in the form
of `deriving(RustcEncodable, RustcDecodable)`.

[![Build Status](https://travis-ci.org/rust-lang/rustc-serialize.svg?branch=master)](https://travis-ci.org/rust-lang/rustc-serialize)

[Documentation](http://doc.rust-lang.org/rustc-serialize)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustc-serialize = "0.2"
```

and this to your crate root:

```rust
extern crate "rustc-serialize" as rustc_serialize;
```
