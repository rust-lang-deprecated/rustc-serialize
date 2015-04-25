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
        if self.is_emitting_map_key { return Err(EncoderError::BadHashmapKey); }
        if let EncodingFormat::Pretty{..} = self.format {
            try!(write!(self.writer, ": "));
        } else {
            try!(write!(self.writer, ":"));
        }
        f(self)
    }
}

