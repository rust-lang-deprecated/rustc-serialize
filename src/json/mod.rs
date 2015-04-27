// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust JSON serialization library
// Copyright (c) 2011 Google Inc.

//! JSON parsing and serialization
//!
//! # What is JSON?
//!
//! JSON (JavaScript Object Notation) is a way to write data in Javascript.
//! Like XML, it allows to encode structured data in a text format that can be
//! easily read by humans Its simple syntax and native compatibility with
//! JavaScript have made it a widely used format.
//!
//! Data types that can be encoded are JavaScript types (see the `Json` enum
//! for more details):
//!
//! * `I64`: equivalent to rust's `i64`
//! * `U64`: equivalent to rust's `u64`
//! * `F64`: equivalent to rust's `f64`
//! * `Boolean`: equivalent to rust's `bool`
//! * `String`: equivalent to rust's `String`
//! * `Array`: equivalent to rust's `Vec<T>`, but also allowing objects of
//!   different types in the
//!   same array
//! * `Object`: equivalent to rust's `BTreeMap<String, json::Json>`
//! * `Null`
//!
//! An object is a series of string keys mapping to values, in `"key": value`
//! format.  Arrays are enclosed in square brackets ([ ... ]) and objects in
//! curly brackets ({ ... }).  A simple JSON document encoding a person,
//! their age, address and phone numbers could look like
//!
//! ```ignore
//! {
//!     "FirstName": "John",
//!     "LastName": "Doe",
//!     "Age": 43,
//!     "Address": {
//!         "Street": "Downing Street 10",
//!         "City": "London",
//!         "Country": "Great Britain"
//!     },
//!     "PhoneNumbers": [
//!         "+44 1234567",
//!         "+44 2345678"
//!     ]
//! }
//! ```
//!
//! # Rust Type-based Encoding and Decoding
//!
//! Rust provides a mechanism for low boilerplate encoding & decoding of values
//! to and from JSON via the serialization API.  To be able to encode a piece
//! of data, it must implement the `rustc_serialize::Encodable` trait.  To be
//! able to decode a piece of data, it must implement the
//! `rustc_serialize::Decodable` trait.  The Rust compiler provides an
//! annotation to automatically generate the code for these traits:
//! `#[derive(RustcDecodable, RustcEncodable)]`
//!
//! The JSON API provides an enum `json::Json` and a trait `ToJson` to encode
//! objects.  The `ToJson` trait provides a `to_json` method to convert an
//! object into a `json::Json` value.  A `json::Json` value can be encoded as a
//! string or buffer using the functions described above.  You can also use the
//! `json::Encoder` object, which implements the `Encoder` trait.
//!
//! When using `ToJson` the `Encodable` trait implementation is not
//! mandatory.
//!
//! # Examples of use
//!
//! ## Using Autoserialization
//!
//! Create a struct called `TestStruct` and serialize and deserialize it to and
//! from JSON using the serialization API, using the derived serialization code.
//!
//! ```rust
//! extern crate rustc_serialize;
//! use rustc_serialize::json;
//!
//! // Automatically generate `RustcDecodable` and `RustcEncodable` trait
//! // implementations
//! #[derive(RustcDecodable, RustcEncodable)]
//! pub struct TestStruct  {
//!     data_int: u8,
//!     data_str: String,
//!     data_vector: Vec<u8>,
//! }
//!
//! fn main() {
//!     let object = TestStruct {
//!         data_int: 1,
//!         data_str: "homura".to_string(),
//!         data_vector: vec![2,3,4,5],
//!     };
//!
//!     // Serialize using `json::encode`
//!     let encoded = json::encode(&object).unwrap();
//!
//!     // Deserialize using `json::decode`
//!     let decoded: TestStruct = json::decode(&encoded).unwrap();
//! }
//! ```
//!
//! ## Using the `ToJson` trait
//!
//! The examples above use the `ToJson` trait to generate the JSON string,
//! which is required for custom mappings.
//!
//! ### Simple example of `ToJson` usage
//!
//! ```rust
//! extern crate rustc_serialize;
//! use rustc_serialize::json::{self, ToJson, Json};
//!
//! // A custom data structure
//! struct ComplexNum {
//!     a: f64,
//!     b: f64,
//! }
//!
//! // JSON value representation
//! impl ToJson for ComplexNum {
//!     fn to_json(&self) -> Json {
//!         Json::String(format!("{}+{}i", self.a, self.b))
//!     }
//! }
//!
//! // Only generate `RustcEncodable` trait implementation
//! #[derive(RustcEncodable)]
//! pub struct ComplexNumRecord {
//!     uid: u8,
//!     dsc: String,
//!     val: Json,
//! }
//!
//! fn main() {
//!     let num = ComplexNum { a: 0.0001, b: 12.539 };
//!     let data: String = json::encode(&ComplexNumRecord{
//!         uid: 1,
//!         dsc: "test".to_string(),
//!         val: num.to_json(),
//!     }).unwrap();
//!     println!("data: {}", data);
//!     // data: {"uid":1,"dsc":"test","val":"0.0001+12.539i"};
//! }
//! ```
//!
//! ### Verbose example of `ToJson` usage
//!
//! ```rust
//! extern crate rustc_serialize;
//! use std::collections::BTreeMap;
//! use rustc_serialize::json::{self, Json, ToJson};
//!
//! // Only generate `Decodable` trait implementation
//! #[derive(RustcDecodable)]
//! pub struct TestStruct {
//!     data_int: u8,
//!     data_str: String,
//!     data_vector: Vec<u8>,
//! }
//!
//! // Specify encoding method manually
//! impl ToJson for TestStruct {
//!     fn to_json(&self) -> Json {
//!         let mut d = BTreeMap::new();
//!         // All standard types implement `to_json()`, so use it
//!         d.insert("data_int".to_string(), self.data_int.to_json());
//!         d.insert("data_str".to_string(), self.data_str.to_json());
//!         d.insert("data_vector".to_string(), self.data_vector.to_json());
//!         Json::Object(d)
//!     }
//! }
//!
//! fn main() {
//!     // Serialize using `ToJson`
//!     let input_data = TestStruct {
//!         data_int: 1,
//!         data_str: "madoka".to_string(),
//!         data_vector: vec![2,3,4,5],
//!     };
//!     let json_obj: Json = input_data.to_json();
//!     let json_str: String = json_obj.to_string();
//!
//!     // Deserialize like before
//!     let decoded: TestStruct = json::decode(&json_str).unwrap();
//! }
//! ```
//!
//! ## Parsing a `str` to `Json` and reading the result
//!
//! ```rust
//! extern crate rustc_serialize;
//! use rustc_serialize::json::Json;
//!
//! fn main() {
//!     let data = Json::from_str("{\"foo\": 13, \"bar\": \"baz\"}").unwrap();
//!     println!("data: {}", data);
//!     // data: {"bar":"baz","foo":13}
//!     println!("object? {}", data.is_object());
//!     // object? true
//!
//!     let obj = data.as_object().unwrap();
//!     let foo = obj.get("foo").unwrap();
//!
//!     println!("array? {:?}", foo.as_array());
//!     // array? None
//!     println!("u64? {:?}", foo.as_u64());
//!     // u64? Some(13u64)
//!
//!     for (key, value) in obj.iter() {
//!         println!("{}: {}", key, match *value {
//!             Json::U64(v) => format!("{} (u64)", v),
//!             Json::String(ref v) => format!("{} (string)", v),
//!             _ => format!("other")
//!         });
//!     }
//!     // bar: baz (string)
//!     // foo: 13 (u64)
//! }
//! ```

pub use self::to_json::ToJson;
pub use self::json::{Json, JsonEvent, Array, Object};
pub use self::error::{ErrorCode, ParserError, error_str};
pub use self::as_json::{AsJson, as_json};
pub use self::builder::{Builder, BuilderError};
pub use self::decoder::{Decoder, decode};
pub use self::encoder::{Encoder, encode};
pub use self::parser::Parser;
pub use self::pretty_json::{PrettyJson, AsPrettyJson, as_pretty_json};
pub use self::stack::{Stack, StackElement};

mod as_json;
mod encoder;
mod pretty_json;
mod test;
mod decoder;
mod to_json;
mod builder;
mod error;
mod json;
mod parser;
mod stack;
