use json::JsonEvent::*;
use json::ErrorCode::*;
use json::ParserError::*;
use json::DecoderError::*;
// use json::ParserState::*;
// use json::InternalStackElement::*;

use std::collections::{HashMap, BTreeMap};
use std::error::Error as StdError;
use std::i64;
use std::io::prelude::*;
use std::mem::swap;
use std::ops::Index;
use std::str::FromStr;
use std::string;
use std::{char, f64, fmt, io, str};

use Encodable;

pub struct AsPrettyJson<'a, T: 'a>
{
    inner: &'a T,
    indent: Option<u32>
}

impl<'a, T> AsPrettyJson<'a, T> {
    /// Set the indentation level for the emitted JSON
    pub fn indent(mut self, indent: u32) -> AsPrettyJson<'a, T> {
        self.indent = Some(indent);
        self
    }
}

impl<'a, T: Encodable> fmt::Display for AsPrettyJson<'a, T> {
    /// Encodes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shim = FormatShim { inner: f };
        let mut encoder = Encoder::new_pretty(&mut shim);
        if let Some(n) = self.indent {
            // unwrap cannot panic for pretty encoders
            let _ = encoder.set_indent(n);
        }
        match self.inner.encode(&mut encoder) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error)
        }
    }
}
