use json::json::{Json, FormatShim};
use json::encoder::Encoder;

use std::fmt;

use Encodable;

pub struct PrettyJson<'a>
{
    inner: &'a Json
}

impl<'a> PrettyJson<'a> {
    pub fn new(content: &'a Json) -> PrettyJson{
        PrettyJson{inner:  content}
    }
}

impl<'a> fmt::Display for PrettyJson<'a> {
    /// Encodes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shim = FormatShim { inner: f };
        let mut encoder = Encoder::new_pretty(&mut shim);
        match self.inner.encode(&mut encoder) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error)
        }
    }
}

pub struct AsPrettyJson<'a, T: 'a>{
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

/// Create an `AsPrettyJson` wrapper which can be used to print a value as JSON
/// on-the-fly via `write!`
pub fn as_pretty_json<T: Encodable>(t: &T) -> AsPrettyJson<T> {
    AsPrettyJson { inner: t, indent: None }
}

