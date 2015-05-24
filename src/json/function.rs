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


/// Shortcut function to decode a JSON `&str` into an object
pub fn decode<T: ::Decodable>(s: &str) -> DecodeResult<T> {
    let json = match Json::from_str(s) {
        Ok(x) => x,
        Err(e) => return Err(ParseError(e))
    };

    let mut decoder = Decoder::new(json);
    ::Decodable::decode(&mut decoder)
}

/// Shortcut function to encode a `T` into a JSON `String`
pub fn encode<T: ::Encodable>(object: &T) -> EncodeResult<string::String> {
    let mut s = String::new();
    {
        let mut encoder = Encoder::new(&mut s);
        try!(object.encode(&mut encoder));
    }
    Ok(s)
}

fn escape_str(wr: &mut fmt::Write, v: &str) -> EncodeResult<()> {
    try!(wr.write_str("\""));

    let mut start = 0;

    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => { continue; }
        };

        if start < i {
            try!(wr.write_str(&v[start..i]));
        }

        try!(wr.write_str(escaped));

        start = i + 1;
    }

    if start != v.len() {
        try!(wr.write_str(&v[start..]));
    }

    try!(wr.write_str("\""));
    Ok(())
}

fn escape_char(writer: &mut fmt::Write, v: char) -> EncodeResult<()> {
    let mut buf = [0; 4];
    let _ = write!(&mut &mut buf[..], "{}", v);
    let buf = unsafe { str::from_utf8_unchecked(&buf[..v.len_utf8()]) };
    escape_str(writer, buf)
}

fn spaces(wr: &mut fmt::Write, n: u32) -> EncodeResult<()> {
    let mut n = n as usize;
    const BUF: &'static str = "                ";

    while n >= BUF.len() {
        try!(wr.write_str(BUF));
        n -= BUF.len();
    }

    if n > 0 {
        try!(wr.write_str(&BUF[..n]));
    }
    Ok(())
}

fn fmt_number_or_null(v: f64) -> string::String {
    use std::num::FpCategory::{Nan, Infinite};

    match v.classify() {
        Nan | Infinite => "null".to_string(),
        _ => {
            let s = v.to_string();
            if s.contains(".") {s} else {s + ".0"}
        }
    }
}

enum EncodingFormat {
    Compact,
    Pretty {
        curr_indent: u32,
        indent: u32
    }
}

/// Create an `AsJson` wrapper which can be used to print a value as JSON
/// on-the-fly via `write!`
pub fn as_json<T: Encodable>(t: &T) -> AsJson<T> {
    AsJson { inner: t }
}

/// Create an `AsPrettyJson` wrapper which can be used to print a value as JSON
/// on-the-fly via `write!`
pub fn as_pretty_json<T: Encodable>(t: &T) -> AsPrettyJson<T> {
    AsPrettyJson { inner: t, indent: None }
}

struct FormatShim<'a, 'b: 'a> {
    inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> fmt::Write for FormatShim<'a, 'b> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.inner.write_str(s) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error)
        }
    }
}

/// Returns a readable error string for a given error code.
pub fn error_str(error: ErrorCode) -> &'static str {
    match error {
        InvalidSyntax => "invalid syntax",
        InvalidNumber => "invalid number",
        EOFWhileParsingObject => "EOF While parsing object",
        EOFWhileParsingArray => "EOF While parsing array",
        EOFWhileParsingValue => "EOF While parsing value",
        EOFWhileParsingString => "EOF While parsing string",
        KeyMustBeAString => "key must be a string",
        ExpectedColon => "expected `:`",
        TrailingCharacters => "trailing characters",
        TrailingComma => "trailing comma",
        InvalidEscape => "invalid escape",
        UnrecognizedHex => "invalid \\u{ esc}ape (unrecognized hex)",
        NotFourDigit => "invalid \\u{ esc}ape (not four digits)",
        ControlCharacterInString => "unescaped control character in string",
        NotUtf8 => "contents not utf-8",
        InvalidUnicodeCodePoint => "invalid Unicode code point",
        LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape",
        UnexpectedEndOfHexEscape => "unexpected end of hex escape",
    }
}
