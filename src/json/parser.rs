use json::JsonEvent;
use json::{ErrorCode, ParserError};
use json::stack::{Stack, SemiPubMethods};

use std::i64;
use std::string;
use std::{char};

/// A streaming JSON parser implemented as an iterator of JsonEvent, consuming
/// an iterator of char.
pub struct Parser<T> {
    rdr: T,
    ch: Option<char>,
    line: usize,
    col: usize,
    // We maintain a stack representing where we are in the logical structure
    // of the JSON stream.
    stack: Stack,
    // A state machine is kept to make it possible to interrupt and resume parsing.
    state: ParserState,
}

impl<T: Iterator<Item = char>> Iterator for Parser<T> {
    type Item = JsonEvent;

    fn next(&mut self) -> Option<JsonEvent> {
        if self.state == ParserState::ParseFinished {
            return None;
        }

        if self.state == ParserState::ParseBeforeFinish {
            self.parse_whitespace();
            // Make sure there is no trailing characters.
            if self.eof() {
                self.state = ParserState::ParseFinished;
                return None;
            } else {
                return Some(self.error_event(ErrorCode::TrailingCharacters));
            }
        }
        return Some(self.parse());
    }
}

impl<T: Iterator<Item = char>> Parser<T> {
    /// Creates the JSON parser.
    pub fn new(rdr: T) -> Parser<T> {
        let mut p = Parser {
            rdr: rdr,
            ch: Some('\x00'),
            line: 1,
            col: 0,
            stack: Stack::new(),
            state: ParserState::ParseStart,
        };
        p.bump();
        return p;
    }

    /// Provides access to the current position in the logical structure of the
    /// JSON stream.
    pub fn stack<'l>(&'l self) -> &'l Stack {
        return &self.stack;
    }

    fn eof(&self) -> bool { self.ch.is_none() }
    fn ch_or_null(&self) -> char { self.ch.unwrap_or('\x00') }
    fn bump(&mut self) {
        self.ch = self.rdr.next();

        if self.ch_is('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.bump();
        self.ch
    }
    fn ch_is(&self, c: char) -> bool {
        self.ch == Some(c)
    }

    pub fn error<E>(&self, reason: ErrorCode) -> Result<E, ParserError> {
        Err(ParserError::SyntaxError(reason, self.line, self.col))
    }

    fn parse_whitespace(&mut self) {
        while self.ch_is(' ') ||
              self.ch_is('\n') ||
              self.ch_is('\t') ||
              self.ch_is('\r') { self.bump(); }
    }

    fn parse_number(&mut self) -> JsonEvent {
        let mut neg = false;

        if self.ch_is('-') {
            self.bump();
            neg = true;
        }

        let res = match self.parse_u64() {
            Ok(res) => res,
            Err(e) => { return JsonEvent::Error(e); }
        };

        if self.ch_is('.') || self.ch_is('e') || self.ch_is('E') {
            let mut res = res as f64;

            if self.ch_is('.') {
                res = match self.parse_decimal(res) {
                    Ok(res) => res,
                    Err(e) => { return JsonEvent::Error(e); }
                };
            }

            if self.ch_is('e') || self.ch_is('E') {
                res = match self.parse_exponent(res) {
                    Ok(res) => res,
                    Err(e) => {
                        return JsonEvent::Error(e);
                    }
                };
            }
            if neg {
                res *= -1.0;
            }
            JsonEvent::F64Value(res)
        } else {
            if neg {
                // Make sure we don't underflow.
                if res > (i64::MAX as u64) + 1 {
                    JsonEvent::Error(ParserError::SyntaxError(
                            ErrorCode::InvalidNumber,
                            self.line,
                            self.col))
                } else {
                    JsonEvent::I64Value((!res + 1) as i64)
                }
            } else {
                JsonEvent::U64Value(res)
            }
        }
    }

    fn parse_u64(&mut self) -> Result<u64, ParserError> {
        let mut accum: u64 = 0;

        match self.ch_or_null() {
            '0' => {
                self.bump();

                // A leading '0' must be the only digit before the decimal point.
                match self.ch_or_null() {
                    '0' ... '9' => return self.error(ErrorCode::InvalidNumber),
                    _ => ()
                }
            },
            '1' ... '9' => {
                while !self.eof() {
                    match self.ch_or_null() {
                        c @ '0' ... '9' => {
                            macro_rules! try_or_invalid {
                                ($e: expr) => {
                                    match $e {
                                        Some(v) => v,
                                        None => return self.error(ErrorCode::InvalidNumber)
                                    }
                                }
                            }
                            accum = try_or_invalid!(accum.checked_mul(10));
                            accum = try_or_invalid!(accum.checked_add((c as u64) - ('0' as u64)));

                            self.bump();
                        }
                        _ => break,
                    }
                }
            }
            _ => return self.error(ErrorCode::InvalidNumber),
        }

        Ok(accum)
    }

    fn parse_decimal(&mut self, mut res: f64) -> Result<f64, ParserError> {
        self.bump();

        // Make sure a digit follows the decimal place.
        match self.ch_or_null() {
            '0' ... '9' => (),
             _ => return self.error(ErrorCode::InvalidNumber)
        }

        let mut dec = 1.0;
        while !self.eof() {
            match self.ch_or_null() {
                c @ '0' ... '9' => {
                    dec /= 10.0;
                    res += (((c as isize) - ('0' as isize)) as f64) * dec;
                    self.bump();
                }
                _ => break,
            }
        }

        Ok(res)
    }

    fn parse_exponent(&mut self, mut res: f64) -> Result<f64, ParserError> {
        self.bump();

        let mut exp = 0;
        let mut neg_exp = false;

        if self.ch_is('+') {
            self.bump();
        } else if self.ch_is('-') {
            self.bump();
            neg_exp = true;
        }

        // Make sure a digit follows the exponent place.
        match self.ch_or_null() {
            '0' ... '9' => (),
            _ => return self.error(ErrorCode::InvalidNumber)
        }
        while !self.eof() {
            match self.ch_or_null() {
                c @ '0' ... '9' => {
                    exp *= 10;
                    exp += (c as usize) - ('0' as usize);

                    self.bump();
                }
                _ => break
            }
        }

        let exp = 10_f64.powi(exp as i32);
        if neg_exp {
            res /= exp;
        } else {
            res *= exp;
        }

        Ok(res)
    }

    fn decode_hex_escape(&mut self) -> Result<u16, ParserError> {
        let mut i = 0;
        let mut n = 0;
        while i < 4 {
            self.bump();
            n = match self.ch_or_null() {
                c @ '0' ... '9' => n * 16 + ((c as u16) - ('0' as u16)),
                c @ 'a' ... 'f' => n * 16 + (10 + (c as u16) - ('a' as u16)),
                c @ 'A' ... 'F' => n * 16 + (10 + (c as u16) - ('A' as u16)),
                _ => return self.error(ErrorCode::InvalidEscape)
            };

            i += 1;
        }

        Ok(n)
    }

    fn parse_str(&mut self) -> Result<string::String, ParserError> {
        let mut escape = false;
        let mut res = string::String::new();

        loop {
            self.bump();
            if self.eof() {
                return self.error(ErrorCode::EOFWhileParsingString);
            }

            if escape {
                match self.ch_or_null() {
                    '"' => res.push('"'),
                    '\\' => res.push('\\'),
                    '/' => res.push('/'),
                    'b' => res.push('\x08'),
                    'f' => res.push('\x0c'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'u' => match try!(self.decode_hex_escape()) {
                        0xDC00 ... 0xDFFF => {
                            return self.error(
                                    ErrorCode::LoneLeadingSurrogateInHexEscape)
                        }

                        // Non-BMP characters are encoded as a sequence of
                        // two hex escapes, representing UTF-16 surrogates.
                        n1 @ 0xD800 ... 0xDBFF => {
                            match (self.next_char(), self.next_char()) {
                                (Some('\\'), Some('u')) => (),
                                _ => return self.error(
                                        ErrorCode::UnexpectedEndOfHexEscape),
                            }

                            let n2 = try!(self.decode_hex_escape());
                            if n2 < 0xDC00 || n2 > 0xDFFF {
                                return self.error(
                                        ErrorCode::LoneLeadingSurrogateInHexEscape)
                            }
                            let c = (((n1 - 0xD800) as u32) << 10 |
                                     (n2 - 0xDC00) as u32) + 0x1_0000;
                            res.push(char::from_u32(c).unwrap());
                        }

                        n => match char::from_u32(n as u32) {
                            Some(c) => res.push(c),
                            None => return self.error(
                                    ErrorCode::InvalidUnicodeCodePoint),
                        },
                    },
                    _ => return self.error(ErrorCode::InvalidEscape),
                }
                escape = false;
            } else if self.ch_is('\\') {
                escape = true;
            } else {
                match self.ch {
                    Some('"') => {
                        self.bump();
                        return Ok(res);
                    },
                    Some(c) if c.is_control() =>
                        return self.error(ErrorCode::ControlCharacterInString),
                    Some(c) => res.push(c),
                    None => unreachable!()
                }
            }
        }
    }

    // Invoked at each iteration, consumes the stream until it has enough
    // information to return a JsonEvent.
    // Manages an internal state so that parsing can be interrupted and resumed.
    // Also keeps track of the position in the logical structure of the json
    // stream int the form of a stack that can be queried by the user using the
    // stack() method.
    fn parse(&mut self) -> JsonEvent {
        loop {
            // The only paths where the loop can spin a new iteration
            // are in the cases ParseArrayComma and ParseObjectComma if ','
            // is parsed. In these cases the state is set to (respectively)
            // ParserState::ParseArray(false) and ParseObject(false),
            // which always return,
            // so there is no risk of getting stuck in an infinite loop.
            // All other paths return before the end of the loop's iteration.
            self.parse_whitespace();

            match self.state {
                ParserState::ParseStart => {
                    return self.parse_start();
                }
                ParserState::ParseArray(first) => {
                    return self.parse_array(first);
                }
                ParserState::ParseArrayComma => {
                    match self.parse_array_comma_or_end() {
                        Some(evt) => { return evt; }
                        None => {}
                    }
                }
                ParserState::ParseObject(first) => {
                    return self.parse_object(first);
                }
                ParserState::ParseObjectComma => {
                    self.stack.pop();
                    if self.ch_is(',') {
                        self.state = ParserState::ParseObject(false);
                        self.bump();
                    } else {
                        return self.parse_object_end();
                    }
                }
                _ => {
                    return self.error_event(ErrorCode::InvalidSyntax);
                }
            }
        }
    }

    fn parse_start(&mut self) -> JsonEvent {
        let val = self.parse_value();
        self.state = match val {
            JsonEvent::Error(_) => ParserState::ParseFinished,
            JsonEvent::ArrayStart => ParserState::ParseArray(true),
            JsonEvent::ObjectStart => ParserState::ParseObject(true),
            _ => ParserState::ParseBeforeFinish,
        };
        return val;
    }

    fn parse_array(&mut self, first: bool) -> JsonEvent {
        if self.ch_is(']') {
            if !first {
                self.error_event(ErrorCode::InvalidSyntax)
            } else {
                self.state = if self.stack.is_empty() {
                    ParserState::ParseBeforeFinish
                } else if self.stack.last_is_index() {
                    ParserState::ParseArrayComma
                } else {
                    ParserState::ParseObjectComma
                };
                self.bump();
                JsonEvent::ArrayEnd
            }
        } else {
            if first {
                self.stack.push_index(0);
            }
            let val = self.parse_value();
            self.state = match val {
                JsonEvent::Error(_) => ParserState::ParseFinished,
                JsonEvent::ArrayStart => ParserState::ParseArray(true),
                JsonEvent::ObjectStart => ParserState::ParseObject(true),
                _ => ParserState::ParseArrayComma,
            };
            val
        }
    }

    fn parse_array_comma_or_end(&mut self) -> Option<JsonEvent> {
        if self.ch_is(',') {
            self.stack.bump_index();
            self.state = ParserState::ParseArray(false);
            self.bump();
            None
        } else if self.ch_is(']') {
            self.stack.pop();
            self.state = if self.stack.is_empty() {
                ParserState::ParseBeforeFinish
            } else if self.stack.last_is_index() {
                ParserState::ParseArrayComma
            } else {
                ParserState::ParseObjectComma
            };
            self.bump();
            Some(JsonEvent::ArrayEnd)
        } else if self.eof() {
            Some(self.error_event(ErrorCode::EOFWhileParsingArray))
        } else {
            Some(self.error_event(ErrorCode::InvalidSyntax))
        }
    }

    fn parse_object(&mut self, first: bool) -> JsonEvent {
        if self.ch_is('}') {
            if !first {
                if self.stack.is_empty() {
                    return self.error_event(ErrorCode::TrailingComma);
                } else {
                    self.stack.pop();
                }
            }
            self.state = if self.stack.is_empty() {
                ParserState::ParseBeforeFinish
            } else if self.stack.last_is_index() {
                ParserState::ParseArrayComma
            } else {
                ParserState::ParseObjectComma
            };
            self.bump();
            return JsonEvent::ObjectEnd;
        }
        if self.eof() {
            return self.error_event(ErrorCode::EOFWhileParsingObject);
        }
        if !self.ch_is('"') {
            return self.error_event(ErrorCode::KeyMustBeAString);
        }
        let s = match self.parse_str() {
            Ok(s) => s,
            Err(e) => {
                self.state = ParserState::ParseFinished;
                return JsonEvent::Error(e);
            }
        };
        self.parse_whitespace();
        if self.eof() {
            return self.error_event(ErrorCode::EOFWhileParsingObject);
        } else if self.ch_or_null() != ':' {
            return self.error_event(ErrorCode::ExpectedColon);
        }
        self.stack.push_key(s);
        self.bump();
        self.parse_whitespace();

        let val = self.parse_value();

        self.state = match val {
            JsonEvent::Error(_) => ParserState::ParseFinished,
            JsonEvent::ArrayStart => ParserState::ParseArray(true),
            JsonEvent::ObjectStart => ParserState::ParseObject(true),
            _ => ParserState::ParseObjectComma,
        };
        return val;
    }

    fn parse_object_end(&mut self) -> JsonEvent {
        if self.ch_is('}') {
            self.state = if self.stack.is_empty() {
                ParserState::ParseBeforeFinish
            } else if self.stack.last_is_index() {
                ParserState::ParseArrayComma
            } else {
                ParserState::ParseObjectComma
            };
            self.bump();
            JsonEvent::ObjectEnd
        } else if self.eof() {
            self.error_event(ErrorCode::EOFWhileParsingObject)
        } else {
            self.error_event(ErrorCode::InvalidSyntax)
        }
    }

    fn parse_value(&mut self) -> JsonEvent {
        if self.eof() {
            return self.error_event(ErrorCode::EOFWhileParsingValue);
        }
        match self.ch_or_null() {
            'n' => { self.parse_ident("ull", JsonEvent::NullValue) }
            't' => { self.parse_ident("rue", JsonEvent::BooleanValue(true))}
            'f' => { self.parse_ident("alse", JsonEvent::BooleanValue(false))}
            '0' ... '9' | '-' => self.parse_number(),
            '"' => match self.parse_str() {
                Ok(s) => JsonEvent::StringValue(s),
                Err(e) => JsonEvent::Error(e),
            },
            '[' => {
                self.bump();
                JsonEvent::ArrayStart
            }
            '{' => {
                self.bump();
                JsonEvent::ObjectStart
            }
            _ => { self.error_event(ErrorCode::InvalidSyntax) }
        }
    }

    fn parse_ident(&mut self, ident: &str, value: JsonEvent) -> JsonEvent {
        if ident.chars().all(|c| Some(c) == self.next_char()) {
            self.bump();
            value
        } else {
            JsonEvent::Error(ParserError::SyntaxError(
                    ErrorCode::InvalidSyntax,
                    self.line,
                    self.col))
        }
    }

    fn error_event(&mut self, reason: ErrorCode) -> JsonEvent {
        self.state = ParserState::ParseFinished;
        JsonEvent::Error(ParserError::SyntaxError(reason, self.line, self.col))
    }
}

#[derive(PartialEq, Debug)]
enum ParserState {
    // Parse a value in an array, true means first element.
    ParseArray(bool),
    // Parse ',' or ']' after an element in an array.
    ParseArrayComma,
    // Parse a key:value in an object, true means first element.
    ParseObject(bool),
    // Parse ',' or ']' after an element in an object.
    ParseObjectComma,
    // Initial state.
    ParseStart,
    // Expecting the stream to end.
    ParseBeforeFinish,
    // Parsing can't continue.
    ParseFinished,
}
