/// A structure to decode JSON to values in rust.
pub struct Decoder {
    stack: Vec<Json>,
}

impl Decoder {
    /// Creates a new decoder instance for decoding the specified JSON value.
    pub fn new(json: Json) -> Decoder {
        Decoder { stack: vec![json] }
    }
}

impl Decoder {
    fn pop(&mut self) -> DecodeResult<Json> {
        match self.stack.pop() {
            Some(s) => Ok(s),
            None => Err(EOF),
        }
    }
}

macro_rules! expect {
    ($e:expr, Null) => ({
        match try!($e) {
            Json::Null => Ok(()),
            other => Err(ExpectedError("Null".to_string(),
                                       format!("{}", other)))
        }
    });
    ($e:expr, $t:ident) => ({
        match try!($e) {
            Json::$t(v) => Ok(v),
            other => {
                Err(ExpectedError(stringify!($t).to_string(),
                                  format!("{}", other)))
            }
        }
    })
}

macro_rules! read_primitive {
    ($name:ident, $ty:ident) => {
        #[allow(unused_comparisons)]
        fn $name(&mut self) -> DecodeResult<$ty> {
            match try!(self.pop()) {
                Json::I64(i) => {
                    let other = i as $ty;
                    if i == other as i64 && (other > 0) == (i > 0) {
                        Ok(other)
                    } else {
                        Err(ExpectedError("Number".to_string(), i.to_string()))
                    }
                }
                Json::U64(u) => {
                    let other = u as $ty;
                    if u == other as u64 && other >= 0 {
                        Ok(other)
                    } else {
                        Err(ExpectedError("Number".to_string(), u.to_string()))
                    }
                }
                Json::F64(f) => {
                    Err(ExpectedError("Integer".to_string(), f.to_string()))
                }
                // re: #12967.. a type w/ numeric keys (ie HashMap<usize, V> etc)
                // is going to have a string here, as per JSON spec.
                Json::String(s) => match s.parse() {
                    Ok(f)  => Ok(f),
                    Err(_) => Err(ExpectedError("Number".to_string(), s)),
                },
                value => {
                    Err(ExpectedError("Number".to_string(), value.to_string()))
                }
            }
        }
    }
}

impl ::Decoder for Decoder {
    type Error = DecoderError;

    fn read_nil(&mut self) -> DecodeResult<()> {
        expect!(self.pop(), Null)
    }

    read_primitive! { read_usize, usize }
    read_primitive! { read_u8, u8 }
    read_primitive! { read_u16, u16 }
    read_primitive! { read_u32, u32 }
    read_primitive! { read_u64, u64 }
    read_primitive! { read_isize, isize }
    read_primitive! { read_i8, i8 }
    read_primitive! { read_i16, i16 }
    read_primitive! { read_i32, i32 }
    read_primitive! { read_i64, i64 }

    fn read_f32(&mut self) -> DecodeResult<f32> {
        self.read_f64().map(|x| x as f32)
    }

    fn read_f64(&mut self) -> DecodeResult<f64> {
        match try!(self.pop()) {
            Json::I64(f) => Ok(f as f64),
            Json::U64(f) => Ok(f as f64),
            Json::F64(f) => Ok(f),
            Json::String(s) => {
                // re: #12967.. a type w/ numeric keys (ie HashMap<usize, V> etc)
                // is going to have a string here, as per JSON spec.
                match s.parse() {
                    Ok(f)  => Ok(f),
                    Err(_) => Err(ExpectedError("Number".to_string(), s)),
                }
            },
            Json::Null => Ok(f64::NAN),
            value => Err(ExpectedError("Number".to_string(), format!("{}", value)))
        }
    }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        expect!(self.pop(), Boolean)
    }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(ExpectedError("single character string".to_string(), format!("{}", s)))
    }

    fn read_str(&mut self) -> DecodeResult<string::String> {
        expect!(self.pop(), String)
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str],
                               mut f: F) -> DecodeResult<T>
        where F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let name = match try!(self.pop()) {
            Json::String(s) => s,
            Json::Object(mut o) => {
                let n = match o.remove(&"variant".to_string()) {
                    Some(Json::String(s)) => s,
                    Some(val) => {
                        return Err(ExpectedError("String".to_string(), format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("variant".to_string()))
                    }
                };
                match o.remove(&"fields".to_string()) {
                    Some(Json::Array(l)) => {
                        for field in l.into_iter().rev() {
                            self.stack.push(field);
                        }
                    },
                    Some(val) => {
                        return Err(ExpectedError("Array".to_string(), format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("fields".to_string()))
                    }
                }
                n
            }
            json => {
                return Err(ExpectedError("String or Object".to_string(), format!("{}", json)))
            }
        };
        let idx = match names.iter().position(|n| *n == name) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        f(self, idx)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T, F>(&mut self,
                                         _name: &str,
                                         idx: usize,
                                         f: F)
                                         -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T, F>(&mut self, _name: &str, _len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        let value = try!(f(self));
        try!(self.pop());
        Ok(value)
    }

    fn read_struct_field<T, F>(&mut self,
                               name: &str,
                               _idx: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        let mut obj = try!(expect!(self.pop(), Object));

        let value = match obj.remove(&name.to_string()) {
            None => {
                // Add a Null and try to parse it as an Option<_>
                // to get None as a default value.
                self.stack.push(Json::Null);
                match f(self) {
                    Ok(x) => x,
                    Err(_) => return Err(MissingFieldError(name.to_string())),
                }
            },
            Some(json) => {
                self.stack.push(json);
                try!(f(self))
            }
        };
        self.stack.push(Json::Object(obj));
        Ok(value)
    }

    fn read_tuple<T, F>(&mut self, tuple_len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq(move |d, len| {
            if len == tuple_len {
                f(d)
            } else {
                Err(ExpectedError(format!("Tuple{}", tuple_len), format!("Tuple{}", len)))
            }
        })
    }

    fn read_tuple_arg<T, F>(&mut self, idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T, F>(&mut self,
                               _name: &str,
                               len: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self,
                                   idx: usize,
                                   f: F)
                                   -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T, F>(&mut self, mut f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, bool) -> DecodeResult<T>,
    {
        match try!(self.pop()) {
            Json::Null => f(self, false),
            value => { self.stack.push(value); f(self, true) }
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let array = try!(expect!(self.pop(), Array));
        let len = array.len();
        for v in array.into_iter().rev() {
            self.stack.push(v);
        }
        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let obj = try!(expect!(self.pop(), Object));
        let len = obj.len();
        for (key, value) in obj.into_iter() {
            self.stack.push(value);
            self.stack.push(Json::String(key));
        }
        f(self, len)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> DecoderError {
        ApplicationError(err.to_string())
    }
}
