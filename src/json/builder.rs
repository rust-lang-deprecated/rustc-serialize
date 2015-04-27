use json::parser::Parser;
use json::json::{Json, JsonEvent};
use json::error::{ErrorCode, ParserError};
use json::stack::StackElement;

use std::collections::BTreeMap;
use std::mem::swap;
use std::string;

// Builder and Parser have the same errors.
pub type BuilderError = ParserError;

/// A Builder consumes a json::Parser to create a generic Json structure.
pub struct Builder<T> {
    parser: Parser<T>,
    token: Option<JsonEvent>,
}

impl<T: Iterator<Item = char>> Builder<T> {
    /// Create a JSON Builder.
    pub fn new(src: T) -> Builder<T> {
        Builder { parser: Parser::new(src), token: None, }
    }

    // Decode a Json value from a Parser.
    pub fn build(&mut self) -> Result<Json, BuilderError> {
        self.bump();
        let result = self.build_value();
        self.bump();
        match self.token.take() {
            None => {}
            Some(JsonEvent::Error(e)) => { return Err(e); }
            ref tok => { panic!("unexpected token {:?}", tok); }
        }
        result
    }

    fn bump(&mut self) {
        self.token = self.parser.next();
    }

    fn build_value(&mut self) -> Result<Json, BuilderError> {
        return match self.token.take() {
            Some(JsonEvent::NullValue) => Ok(Json::Null),
            Some(JsonEvent::I64Value(n)) => Ok(Json::I64(n)),
            Some(JsonEvent::U64Value(n)) => Ok(Json::U64(n)),
            Some(JsonEvent::F64Value(n)) => Ok(Json::F64(n)),
            Some(JsonEvent::BooleanValue(b)) => Ok(Json::Boolean(b)),
            Some(JsonEvent::StringValue(ref mut s)) => {
                let mut temp = string::String::new();
                swap(s, &mut temp);
                Ok(Json::String(temp))
            }
            Some(JsonEvent::Error(e)) => Err(e),
            Some(JsonEvent::ArrayStart) => self.build_array(),
            Some(JsonEvent::ObjectStart) => self.build_object(),
            Some(JsonEvent::ObjectEnd) => self.parser.error(ErrorCode::InvalidSyntax),
            Some(JsonEvent::ArrayEnd) => self.parser.error(ErrorCode::InvalidSyntax),
            None => self.parser.error(ErrorCode::EOFWhileParsingValue),
        }
    }

    fn build_array(&mut self) -> Result<Json, BuilderError> {
        self.bump();
        let mut values = Vec::new();

        loop {
            if let Some(JsonEvent::ArrayEnd) = self.token {
                return Ok(Json::Array(values.into_iter().collect()));
            }
            match self.build_value() {
                Ok(v) => values.push(v),
                Err(e) => { return Err(e) }
            }
            self.bump();
        }
    }

    fn build_object(&mut self) -> Result<Json, BuilderError> {
        self.bump();

        let mut values = BTreeMap::new();

        loop {
            match self.token.take() {
                Some(JsonEvent::ObjectEnd) => {
                    return Ok(Json::Object(values));
                }
                Some(JsonEvent::Error(e)) => {
                    return Err(e);
                }
                None => {
                    break;
                }
                token => {
                    self.token = token;
                }
            }
            let key = match self.parser.stack().top() {
                Some(StackElement::Key(k)) => { k.to_string() }
                _ => { panic!("invalid state"); }
            };
            match self.build_value() {
                Ok(value) => { values.insert(key, value); }
                Err(e) => { return Err(e); }
            }
            self.bump();
        }
        return self.parser.error(ErrorCode::EOFWhileParsingObject);
    }
}
