// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// ignore-lexer-test FIXME #15679

//! Base64 binary-to-text encoding

pub use self::FromBase64Error::*;
pub use self::CharacterSet::*;

use std::fmt;
use std::error;

/// Available encoding character sets
#[derive(Copy)]
pub enum CharacterSet {
    /// The standard character set (uses `+` and `/`)
    Standard,
    /// The URL safe character set (uses `-` and `_`)
    UrlSafe
}

/// Available newline types
#[derive(Copy)]
pub enum Newline {
    /// A linefeed (i.e. Unix-style newline)
    LF,
    /// A carriage return and a linefeed (i.e. Windows-style newline)
    CRLF
}

/// Contains configuration parameters for `to_base64`.
#[derive(Copy)]
pub struct Config {
    /// Character set to use
    pub char_set: CharacterSet,
    /// Newline to use
    pub newline: Newline,
    /// True to pad output with `=` characters
    pub pad: bool,
    /// `Some(len)` to wrap lines at `len`, `None` to disable line wrapping
    pub line_length: Option<uint>
}

/// Configuration for RFC 4648 standard base64 encoding
pub static STANDARD: Config =
    Config {char_set: Standard, newline: Newline::CRLF, pad: true, line_length: None};

/// Configuration for RFC 4648 base64url encoding
pub static URL_SAFE: Config =
    Config {char_set: UrlSafe, newline: Newline::CRLF, pad: false, line_length: None};

/// Configuration for RFC 2045 MIME base64 encoding
pub static MIME: Config =
    Config {char_set: Standard, newline: Newline::CRLF, pad: true, line_length: Some(76)};

static STANDARD_CHARS: &'static[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                        abcdefghijklmnopqrstuvwxyz\
                                        0123456789+/";

static URLSAFE_CHARS: &'static[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                       abcdefghijklmnopqrstuvwxyz\
                                       0123456789-_";

/// A trait for converting a value to base64 encoding.
pub trait ToBase64 {
    /// Converts the value of `self` to a base64 value following the specified
    /// format configuration, returning the owned string.
    fn to_base64(&self, config: Config) -> String;
}

impl ToBase64 for [u8] {
    /// Turn a vector of `u8` bytes into a base64 string.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![allow(staged_unstable)]
    /// extern crate "rustc-serialize" as rustc_serialize;
    /// use rustc_serialize::base64::{ToBase64, STANDARD};
    ///
    /// fn main () {
    ///     let str = [52,32].to_base64(STANDARD);
    ///     println!("base 64 output: {:?}", str);
    /// }
    /// ```
    fn to_base64(&self, config: Config) -> String {
        let bytes = match config.char_set {
            Standard => STANDARD_CHARS,
            UrlSafe => URLSAFE_CHARS
        };

        // In general, this Vec only needs (4/3) * self.len() memory, but
        // addition is faster than multiplication and division.
        let mut v = Vec::with_capacity(self.len() + self.len());
        let mut i = 0;
        let mut cur_length = 0;
        let len = self.len();
        let mod_len = len % 3;
        let cond_len = len - mod_len;
        let newline = match config.newline {
            Newline::LF => b"\n",
            Newline::CRLF => b"\r\n"
        };
        while i < cond_len {
            let (first, second, third) = (self[i], self[i + 1], self[i + 2]);
            if let Some(line_length) = config.line_length {
                if cur_length >= line_length {
                    v.push_all(newline);
                    cur_length = 0;
                }
            }

            let n = (first  as u32) << 16 |
                    (second as u32) << 8 |
                    (third  as u32);

            // This 24-bit number gets separated into four 6-bit numbers.
            v.push(bytes[((n >> 18) & 63) as uint]);
            v.push(bytes[((n >> 12) & 63) as uint]);
            v.push(bytes[((n >> 6 ) & 63) as uint]);
            v.push(bytes[(n & 63) as uint]);

            cur_length += 4;
            i += 3;
        }

        if mod_len != 0 {
            if let Some(line_length) = config.line_length {
                if cur_length >= line_length {
                    v.push_all(newline);
                }
            }
        }

        // Heh, would be cool if we knew this was exhaustive
        // (the dream of bounded integer types)
        match mod_len {
            0 => (),
            1 => {
                let n = (self[i] as u32) << 16;
                v.push(bytes[((n >> 18) & 63) as uint]);
                v.push(bytes[((n >> 12) & 63) as uint]);
                if config.pad {
                    v.push(b'=');
                    v.push(b'=');
                }
            }
            2 => {
                let n = (self[i] as u32) << 16 |
                    (self[i + 1u] as u32) << 8;
                v.push(bytes[((n >> 18) & 63) as uint]);
                v.push(bytes[((n >> 12) & 63) as uint]);
                v.push(bytes[((n >> 6 ) & 63) as uint]);
                if config.pad {
                    v.push(b'=');
                }
            }
            _ => panic!("Algebra is broken, please alert the math police")
        }

        unsafe { String::from_utf8_unchecked(v) }
    }
}

/// A trait for converting from base64 encoded values.
pub trait FromBase64 {
    /// Converts the value of `self`, interpreted as base64 encoded data, into
    /// an owned vector of bytes, returning the vector.
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error>;
}

/// Errors that can occur when decoding a base64 encoded string
#[derive(Copy)]
pub enum FromBase64Error {
    /// The input contained a character not part of the base64 format
    InvalidBase64Byte(u8, uint),
    /// The input had an invalid length
    InvalidBase64Length,
}

impl fmt::Show for FromBase64Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidBase64Byte(ch, idx) =>
                write!(f, "Invalid character '{}' at position {}", ch, idx),
            InvalidBase64Length => write!(f, "Invalid length"),
        }
    }
}

impl error::Error for FromBase64Error {
    fn description(&self) -> &str {
        match *self {
            InvalidBase64Byte(_, _) => "invalid character",
            InvalidBase64Length => "invalid length",
        }
    }

    fn detail(&self) -> Option<String> {
        Some(format!("{:?}", self))
    }
}

impl FromBase64 for str {
    /// Convert any base64 encoded string (literal, `@`, `&`, or `~`)
    /// to the byte values it encodes.
    ///
    /// You can use the `String::from_utf8` function to turn a `Vec<u8>` into a
    /// string with characters corresponding to those values.
    ///
    /// # Example
    ///
    /// This converts a string literal to base64 and back.
    ///
    /// ```rust
    /// # #![allow(staged_unstable)]
    /// # #![allow(staged_experimental)]
    /// extern crate "rustc-serialize" as rustc_serialize;
    /// use rustc_serialize::base64::{ToBase64, FromBase64, STANDARD};
    ///
    /// fn main () {
    ///     let hello_str = b"Hello, World".to_base64(STANDARD);
    ///     println!("base64 output: {}", hello_str);
    ///     let res = hello_str.as_slice().from_base64();
    ///     if res.is_ok() {
    ///       let opt_bytes = String::from_utf8(res.unwrap());
    ///       if opt_bytes.is_ok() {
    ///         println!("decoded from base64: {:?}", opt_bytes.unwrap());
    ///       }
    ///     }
    /// }
    /// ```
    #[inline]
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error> {
        self.as_bytes().from_base64()
    }
}

impl FromBase64 for [u8] {
    fn from_base64(&self) -> Result<Vec<u8>, FromBase64Error> {
        let mut r = Vec::with_capacity(self.len());
        let mut buf: u32 = 0;
        let mut modulus = 0i;

        let mut it = self.iter().enumerate();
        for (idx, &byte) in it {
            let val = byte as u32;

            match byte {
                b'A'...b'Z' => buf |= val - 0x41,
                b'a'...b'z' => buf |= val - 0x47,
                b'0'...b'9' => buf |= val + 0x04,
                b'+' | b'-' => buf |= 0x3E,
                b'/' | b'_' => buf |= 0x3F,
                b'\r' | b'\n' => continue,
                b'=' => break,
                _ => return Err(InvalidBase64Byte(self[idx], idx)),
            }

            buf <<= 6;
            modulus += 1;
            if modulus == 4 {
                modulus = 0;
                r.push((buf >> 22) as u8);
                r.push((buf >> 14) as u8);
                r.push((buf >> 6 ) as u8);
            }
        }

        for (idx, &byte) in it {
            match byte {
                b'=' | b'\r' | b'\n' => continue,
                _ => return Err(InvalidBase64Byte(self[idx], idx)),
            }
        }

        match modulus {
            2 => {
                r.push((buf >> 10) as u8);
            }
            3 => {
                r.push((buf >> 16) as u8);
                r.push((buf >> 8 ) as u8);
            }
            0 => (),
            _ => return Err(InvalidBase64Length),
        }

        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use base64::{Config, Newline, FromBase64, ToBase64, STANDARD, URL_SAFE};

    #[test]
    fn test_to_base64_basic() {
        assert_eq!("".as_bytes().to_base64(STANDARD), "");
        assert_eq!("f".as_bytes().to_base64(STANDARD), "Zg==");
        assert_eq!("fo".as_bytes().to_base64(STANDARD), "Zm8=");
        assert_eq!("foo".as_bytes().to_base64(STANDARD), "Zm9v");
        assert_eq!("foob".as_bytes().to_base64(STANDARD), "Zm9vYg==");
        assert_eq!("fooba".as_bytes().to_base64(STANDARD), "Zm9vYmE=");
        assert_eq!("foobar".as_bytes().to_base64(STANDARD), "Zm9vYmFy");
    }

    #[test]
    fn test_to_base64_crlf_line_break() {
        assert!(![0u8; 1000].to_base64(Config {line_length: None, ..STANDARD})
                              .contains("\r\n"));
        assert_eq!(b"foobar".to_base64(Config {line_length: Some(4),
                                               ..STANDARD}),
                   "Zm9v\r\nYmFy");
    }

    #[test]
    fn test_to_base64_lf_line_break() {
        assert!(![0u8; 1000].to_base64(Config {line_length: None,
                                                 newline: Newline::LF,
                                                 ..STANDARD})
                              .as_slice()
                              .contains("\n"));
        assert_eq!(b"foobar".to_base64(Config {line_length: Some(4),
                                               newline: Newline::LF,
                                               ..STANDARD}),
                   "Zm9v\nYmFy");
    }

    #[test]
    fn test_to_base64_padding() {
        assert_eq!("f".as_bytes().to_base64(Config {pad: false, ..STANDARD}), "Zg");
        assert_eq!("fo".as_bytes().to_base64(Config {pad: false, ..STANDARD}), "Zm8");
    }

    #[test]
    fn test_to_base64_url_safe() {
        assert_eq!([251, 255].to_base64(URL_SAFE), "-_8");
        assert_eq!([251, 255].to_base64(STANDARD), "+/8=");
    }

    #[test]
    fn test_from_base64_basic() {
        assert_eq!("".from_base64().unwrap(), b"");
        assert_eq!("Zg==".from_base64().unwrap(), b"f");
        assert_eq!("Zm8=".from_base64().unwrap(), b"fo");
        assert_eq!("Zm9v".from_base64().unwrap(), b"foo");
        assert_eq!("Zm9vYg==".from_base64().unwrap(), b"foob");
        assert_eq!("Zm9vYmE=".from_base64().unwrap(), b"fooba");
        assert_eq!("Zm9vYmFy".from_base64().unwrap(), b"foobar");
    }

    #[test]
    fn test_from_base64_bytes() {
        assert_eq!(b"Zm9vYmFy".from_base64().unwrap(), b"foobar");
    }

    #[test]
    fn test_from_base64_newlines() {
        assert_eq!("Zm9v\r\nYmFy".from_base64().unwrap(),
                   b"foobar");
        assert_eq!("Zm9vYg==\r\n".from_base64().unwrap(),
                   b"foob");
        assert_eq!("Zm9v\nYmFy".from_base64().unwrap(),
                   b"foobar");
        assert_eq!("Zm9vYg==\n".from_base64().unwrap(),
                   b"foob");
    }

    #[test]
    fn test_from_base64_urlsafe() {
        assert_eq!("-_8".from_base64().unwrap(), "+/8=".from_base64().unwrap());
    }

    #[test]
    fn test_from_base64_invalid_char() {
        assert!("Zm$=".from_base64().is_err());
        assert!("Zg==$".from_base64().is_err());
    }

    #[test]
    fn test_from_base64_invalid_padding() {
        assert!("Z===".from_base64().is_err());
    }

    #[test]
    fn test_base64_random() {
        use std::rand::{thread_rng, Rng};

        for _ in range(0u, 1000) {
            let times = thread_rng().gen_range(1u, 100);
            let v = thread_rng().gen_iter::<u8>().take(times)
                                .collect::<Vec<_>>();
            assert_eq!(v.to_base64(STANDARD)
                        .from_base64()
                        .unwrap(),
                       v);
        }
    }

    #[bench]
    pub fn bench_to_base64(b: &mut Bencher) {
        let s = "イロハニホヘト チリヌルヲ ワカヨタレソ ツネナラム \
                 ウヰノオクヤマ ケフコエテ アサキユメミシ ヱヒモセスン";
        b.iter(|| {
            s.as_bytes().to_base64(STANDARD);
        });
        b.bytes = s.len() as u64;
    }

    #[bench]
    pub fn bench_from_base64(b: &mut Bencher) {
        let s = "イロハニホヘト チリヌルヲ ワカヨタレソ ツネナラム \
                 ウヰノオクヤマ ケフコエテ アサキユメミシ ヱヒモセスン";
        let sb = s.as_bytes().to_base64(STANDARD);
        b.iter(|| {
            sb.from_base64().unwrap();
        });
        b.bytes = sb.len() as u64;
    }

}
