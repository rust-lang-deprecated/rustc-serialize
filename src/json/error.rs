use std::error::Error as StdError;
use std::string;
use std::{fmt, io};

/// The errors that can arise while parsing a JSON stream.
#[derive(Clone, Copy, PartialEq)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidNumber,
    EOFWhileParsingObject,
    EOFWhileParsingArray,
    EOFWhileParsingValue,
    EOFWhileParsingString,
    KeyMustBeAString,
    ExpectedColon,
    TrailingCharacters,
    TrailingComma,
    InvalidEscape,
    InvalidUnicodeCodePoint,
    LoneLeadingSurrogateInHexEscape,
    UnexpectedEndOfHexEscape,
    UnrecognizedHex,
    NotFourDigit,
    ControlCharacterInString,
    NotUtf8,
}

#[derive(Debug)]
pub enum ParserError {
    /// msg, line, col
    SyntaxError(ErrorCode, usize, usize),
    IoError(io::Error),
}

impl PartialEq for ParserError {
    fn eq(&self, other: &ParserError) -> bool {
        match (self, other) {
            (&ParserError::SyntaxError(msg0, line0, col0), &ParserError::SyntaxError(msg1, line1, col1)) =>
                msg0 == msg1 && line0 == line1 && col0 == col1,
            (&ParserError::IoError(_), _) => false,
            (_, &ParserError::IoError(_)) => false,
        }
    }
}

/// Returns a readable error string for a given error code.
pub fn error_str(error: ErrorCode) -> &'static str {
    match error {
        ErrorCode::InvalidSyntax => "invalid syntax",
        ErrorCode::InvalidNumber => "invalid number",
        ErrorCode::EOFWhileParsingObject => "EOF While parsing object",
        ErrorCode::EOFWhileParsingArray => "EOF While parsing array",
        ErrorCode::EOFWhileParsingValue => "EOF While parsing value",
        ErrorCode::EOFWhileParsingString => "EOF While parsing string",
        ErrorCode::KeyMustBeAString => "key must be a string",
        ErrorCode::ExpectedColon => "expected `:`",
        ErrorCode::TrailingCharacters => "trailing characters",
        ErrorCode::TrailingComma => "trailing comma",
        ErrorCode::InvalidEscape => "invalid escape",
        ErrorCode::UnrecognizedHex => "invalid \\u{ esc}ape (unrecognized hex)",
        ErrorCode::NotFourDigit => "invalid \\u{ esc}ape (not four digits)",
        ErrorCode::ControlCharacterInString => "unescaped control character in string",
        ErrorCode::NotUtf8 => "contents not utf-8",
        ErrorCode::InvalidUnicodeCodePoint => "invalid Unicode code point",
        ErrorCode::LoneLeadingSurrogateInHexEscape => "lone leading surrogate in hex escape",
        ErrorCode::UnexpectedEndOfHexEscape => "unexpected end of hex escape",
    }
}

#[derive(PartialEq, Debug)]
pub enum DecoderError {
    ParseError(ParserError),
    ExpectedError(string::String, string::String),
    MissingFieldError(string::String),
    UnknownVariantError(string::String),
    ApplicationError(string::String),
    EOF,
}

#[derive(Copy, Debug)]
pub enum EncoderError {
    FmtError(fmt::Error),
    BadHashmapKey,
}

impl Clone for EncoderError {
    fn clone(&self) -> Self { *self }
}

impl fmt::Debug for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        error_str(*self).fmt(f)
    }
}

impl StdError for DecoderError {
    fn description(&self) -> &str { "decoder error" }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            DecoderError::ParseError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<ParserError> for DecoderError {
    fn from(err: ParserError) -> DecoderError {
        DecoderError::ParseError(From::from(err))
    }
}

impl StdError for ParserError {
    fn description(&self) -> &str { "failed to parse json" }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<io::Error> for ParserError {
    fn from(err: io::Error) -> ParserError {
        ParserError::IoError(err)
    }
}

impl StdError for EncoderError {
    fn description(&self) -> &str { "encoder error" }
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<fmt::Error> for EncoderError {
    fn from(err: fmt::Error) -> EncoderError { EncoderError::FmtError(err) }
}
