use json::json::FormatShim;
use json::encoder::Encoder;

use std::fmt;

use Encodable;

pub struct AsJson<'a, T: 'a>
{
    inner: &'a T
}

impl<'a, T: Encodable> fmt::Display for AsJson<'a, T> {
    /// Encodes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shim = FormatShim { inner: f };
        let mut encoder = Encoder::new(&mut shim);
        match self.inner.encode(&mut encoder) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error)
        }
    }
}

/// Create an `AsJson` wrapper which can be used to print a value as JSON
/// on-the-fly via `write!`
pub fn as_json<T: Encodable>(t: &T) -> AsJson<T> {
    AsJson { inner: t }
}
