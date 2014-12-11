# rustc-serialize

Serialization and deserialization support provided by the compiler in the form
of `deriving(RustcEncodable, RustcDecodable)`.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustc-serialize = "*"
```

and this to your crate root:

```rust
extern crate "rustc-serialize" as rustc_serialize;
```
