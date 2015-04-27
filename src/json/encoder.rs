use json::error::EncoderError;

use std::io::prelude::*;
use std::{fmt, str, string};

pub type EncodeResult<T> = Result<T, EncoderError>;

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

pub fn escape_char(writer: &mut fmt::Write, v: char) -> EncodeResult<()> {
    let mut buf = [0; 4];
    let _ = write!(&mut &mut buf[..], "{}", v);
    let buf = unsafe { str::from_utf8_unchecked(&buf[..v.len_utf8()]) };
    escape_str(writer, buf)
}

pub fn spaces(wr: &mut fmt::Write, n: u32) -> EncodeResult<()> {
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

pub fn fmt_number_or_null(v: f64) -> string::String {
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

/// A structure for implementing serialization to JSON.
pub struct Encoder<'a> {
    writer: &'a mut (fmt::Write+'a),
    format : EncodingFormat,
    is_emitting_map_key: bool,
}

impl<'a> Encoder<'a> {
    /// Creates a new encoder whose output will be written in human-readable
    /// JSON to the specified writer
    pub fn new_pretty(writer: &'a mut fmt::Write) -> Encoder<'a> {
        Encoder {
            writer: writer,
            format: EncodingFormat::Pretty {
                curr_indent: 0,
                indent: 2,
            },
            is_emitting_map_key: false,
        }
    }

    /// Creates a new encoder whose output will be written in compact
    /// JSON to the specified writer
    pub fn new(writer: &'a mut fmt::Write) -> Encoder<'a> {
        Encoder {
            writer: writer,
            format: EncodingFormat::Compact,
            is_emitting_map_key: false,
        }
    }

    /// Set the number of spaces to indent for each level.
    /// This is safe to set during encoding.
    pub fn set_indent(&mut self, new_indent: u32) -> Result<(), ()> {
        if let EncodingFormat::Pretty{ref mut curr_indent, ref mut indent} = self.format {
            // self.indent very well could be 0 so we need to use checked division.
            let level = curr_indent.checked_div(*indent).unwrap_or(0);
            *indent = new_indent;
            *curr_indent = level * *indent;
            Ok(())
        } else {
            Err(())
        }
    }
}

macro_rules! emit_enquoted_if_mapkey {
    ($enc:ident,$e:expr) => {
        if $enc.is_emitting_map_key {
            try!(write!($enc.writer, "\"{}\"", $e));
            Ok(())
        } else {
            try!(write!($enc.writer, "{}", $e));
            Ok(())
        }
    }
}

impl<'a> ::Encoder for Encoder<'a> {
    type Error = EncoderError;

    fn emit_nil(&mut self) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        try!(write!(self.writer, "null"));
        Ok(())
    }

    fn emit_usize(&mut self, v: usize) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }

    fn emit_isize(&mut self, v: isize) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i64(&mut self, v: i64) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i32(&mut self, v: i32) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i16(&mut self, v: i16) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }
    fn emit_i8(&mut self, v: i8) -> EncodeResult<()> { emit_enquoted_if_mapkey!(self, v) }

    fn emit_bool(&mut self, v: bool) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if v {
            try!(write!(self.writer, "true"));
        } else {
            try!(write!(self.writer, "false"));
        }
        Ok(())
    }

    fn emit_f64(&mut self, v: f64) -> EncodeResult<()> {
        emit_enquoted_if_mapkey!(self, fmt_number_or_null(v))
    }
    fn emit_f32(&mut self, v: f32) -> EncodeResult<()> {
        self.emit_f64(v as f64)
    }

    fn emit_char(&mut self, v: char) -> EncodeResult<()> {
        escape_char(self.writer, v)
    }
    fn emit_str(&mut self, v: &str) -> EncodeResult<()> {
        escape_str(self.writer, v)
    }

    fn emit_enum<F>(&mut self, _name: &str, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        f(self)
    }

    fn emit_enum_variant<F>(&mut self,
                            name: &str,
                            _id: usize,
                            cnt: usize,
                            f: F)
                            -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        // enums are encoded as strings or objects
        // Bunny => "Bunny"
        // Kangaroo(34,"William") => {"variant": "Kangaroo", "fields": [34,"William"]}
        if cnt == 0 {
            escape_str(self.writer, name)
        } else {
            if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                try!(write!(self.writer, "{{\n"));
                *curr_indent += indent;
                try!(spaces(self.writer, *curr_indent));
                try!(write!(self.writer, "\"variant\": "));
                try!(escape_str(self.writer, name));
                try!(write!(self.writer, ",\n"));
                try!(spaces(self.writer, *curr_indent));
                try!(write!(self.writer, "\"fields\": [\n"));
                *curr_indent += indent;
            } else {
                try!(write!(self.writer, "{{\"variant\":"));
                try!(escape_str(self.writer, name));
                try!(write!(self.writer, ",\"fields\":["));
            }
            try!(f(self));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent -= indent;
                try!(write!(self.writer, "\n"));
                try!(spaces(self.writer, *curr_indent));
                *curr_indent -= indent;
                try!(write!(self.writer, "]\n"));
                try!(spaces(self.writer, *curr_indent));
                try!(write!(self.writer, "}}"));
            } else {
                try!(write!(self.writer, "]}}"));
            }
            Ok(())
        }
    }

    fn emit_enum_variant_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, ","));
            if let EncodingFormat::Pretty{..} = self.format {
                try!(write!(self.writer, "\n"));
            }
        }
        if let EncodingFormat::Pretty{curr_indent, ..} = self.format {
            try!(spaces(self.writer, curr_indent));
        }
        f(self)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   name: &str,
                                   id: usize,
                                   cnt: usize,
                                   f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         idx: usize,
                                         f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_enum_variant_arg(idx, f)
    }


    fn emit_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if len == 0 {
            try!(write!(self.writer, "{{}}"));
        } else {
            try!(write!(self.writer, "{{"));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent += indent;
            }
            try!(f(self));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent -= indent;
                try!(write!(self.writer, "\n"));
                try!(spaces(self.writer, *curr_indent));
            }
            try!(write!(self.writer, "}}"));
        }
        Ok(())
    }

    fn emit_struct_field<F>(&mut self, name: &str, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, ","));
        }
        if let EncodingFormat::Pretty{curr_indent, ..} = self.format {
            try!(write!(self.writer, "\n"));
            try!(spaces(self.writer, curr_indent));
        }
        try!(escape_str(self.writer, name));
        if let EncodingFormat::Pretty{..} = self.format {
            try!(write!(self.writer, ": "));
        } else {
            try!(write!(self.writer, ":"));
        }
        f(self)
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_seq_elt(idx, f)
    }

    fn emit_option<F>(&mut self, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodeResult<()> {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        self.emit_nil()
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        f(self)
    }

    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if len == 0 {
            try!(write!(self.writer, "[]"));
        } else {
            try!(write!(self.writer, "["));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent += indent;
            }
            try!(f(self));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent -= indent;
                try!(write!(self.writer, "\n"));
                try!(spaces(self.writer, *curr_indent));
            }
            try!(write!(self.writer, "]"));
        }
        Ok(())
    }

    fn emit_seq_elt<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, ","));
        }
        if let EncodingFormat::Pretty{ref mut curr_indent, ..} = self.format {
            try!(write!(self.writer, "\n"));
            try!(spaces(self.writer, *curr_indent));
        }
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if len == 0 {
            try!(write!(self.writer, "{{}}"));
        } else {
            try!(write!(self.writer, "{{"));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent += indent;
            }
            try!(f(self));
            if let EncodingFormat::Pretty{ref mut curr_indent, indent} = self.format {
                *curr_indent -= indent;
                try!(write!(self.writer, "\n"));
                try!(spaces(self.writer, *curr_indent));
            }
            try!(write!(self.writer, "}}"));
        }
        Ok(())
    }

    fn emit_map_elt_key<F>(&mut self, idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if idx != 0 {
            try!(write!(self.writer, ","));
        }
        if let EncodingFormat::Pretty{curr_indent, ..} = self.format {
            try!(write!(self.writer, "\n"));
            try!(spaces(self.writer, curr_indent));
        }
        self.is_emitting_map_key = true;
        try!(f(self));
        self.is_emitting_map_key = false;
        Ok(())
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> EncodeResult<()> where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult<()>,
    {
        if self.is_emitting_map_key {
            return Err(EncoderError::BadHashmapKey);
        }
        if let EncodingFormat::Pretty{..} = self.format {
            try!(write!(self.writer, ": "));
        } else {
            try!(write!(self.writer, ":"));
        }
        f(self)
    }
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
