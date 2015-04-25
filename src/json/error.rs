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
            (&SyntaxError(msg0, line0, col0), &SyntaxError(msg1, line1, col1)) =>
                msg0 == msg1 && line0 == line1 && col0 == col1,
            (&IoError(_), _) => false,
            (_, &IoError(_)) => false,
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

// Builder and Parser have the same errors.
pub type BuilderError = ParserError;

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

fn io_error_to_error(err: io::Error) -> ParserError {
    IoError(err)
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

impl StdError for ParserError {
    fn description(&self) -> &str { "failed to parse json" }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
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

pub type EncodeResult<T> = Result<T, EncoderError>;
pub type DecodeResult<T> = Result<T, DecoderError>;
