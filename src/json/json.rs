/// Represents a json value
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Json {
    I64(i64),
    U64(u64),
    F64(f64),
    String(string::String),
    Boolean(bool),
    Array(self::Array),
    Object(self::Object),
    Null,
}

impl Json {
    /// Decodes a json value from an `&mut io::Read`
    pub fn from_reader(rdr: &mut io::Read) -> Result<Self, BuilderError> {
        let contents = {
            let mut c = Vec::new();
            match rdr.read_to_end(&mut c) {
                Ok(_)  => (),
                Err(e) => return Err(io_error_to_error(e))
            }
            c
        };
        let s = match str::from_utf8(&contents).ok() {
            Some(s) => s,
            _       => return Err(SyntaxError(NotUtf8, 0, 0))
        };
        let mut builder = Builder::new(s.chars());
        builder.build()
    }

    /// Decodes a json value from a string
    pub fn from_str(s: &str) -> Result<Self, BuilderError> {
        let mut builder = Builder::new(s.chars());
        builder.build()
    }

    /// Borrow this json object as a pretty object to generate a pretty
    /// representation for it via `Display`.
    pub fn pretty(&self) -> PrettyJson {
        PrettyJson { inner: self }
    }

     /// If the Json value is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &str) -> Option<&'a Json>{
        match self {
            &Json::Object(ref map) => map.get(key),
            _ => None
        }
    }

    /// Attempts to get a nested Json Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the Json value associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&str]) -> Option<&'a Json>{
        let mut target = self;
        for key in keys.iter() {
            match target.find(*key) {
                Some(t) => { target = t; },
                None => return None
            }
        }
        Some(target)
    }

    /// If the Json value is an Object, performs a depth-first search until
    /// a value associated with the provided key is found. If no value is found
    /// or the Json value is not an Object, returns None.
    pub fn search<'a>(&'a self, key: &str) -> Option<&'a Json> {
        match self {
            &Json::Object(ref map) => {
                match map.get(key) {
                    Some(json_value) => Some(json_value),
                    None => {
                        for (_, v) in map.iter() {
                            match v.search(key) {
                                x if x.is_some() => return x,
                                _ => ()
                            }
                        }
                        None
                    }
                }
            },
            _ => None
        }
    }

    /// Returns true if the Json value is an Object. Returns false otherwise.
    pub fn is_object<'a>(&'a self) -> bool {
        self.as_object().is_some()
    }

    /// If the Json value is an Object, returns the associated BTreeMap.
    /// Returns None otherwise.
    pub fn as_object<'a>(&'a self) -> Option<&'a Object> {
        match self {
            &Json::Object(ref map) => Some(map),
            _ => None
        }
    }

    /// If the Json value is an Object, returns the associated mutable BTreeMap.
    /// Returns None otherwise.
    pub fn as_object_mut<'a>(&'a mut self) -> Option<&'a mut Object> {
        match self {
            &mut Json::Object(ref mut map) => Some(map),
            _ => None
        }
    }

    /// Returns true if the Json value is an Array. Returns false otherwise.
    pub fn is_array<'a>(&'a self) -> bool {
        self.as_array().is_some()
    }

    /// If the Json value is an Array, returns the associated vector.
    /// Returns None otherwise.
    pub fn as_array<'a>(&'a self) -> Option<&'a Array> {
        match self {
            &Json::Array(ref array) => Some(&*array),
            _ => None
        }
    }

    /// If the Json value is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut<'a>(&'a mut self) -> Option<&'a mut Array> {
        match self {
            &mut Json::Array(ref mut list) => Some(list),
            _ => None
        }
    }

    /// Returns true if the Json value is a String. Returns false otherwise.
    pub fn is_string<'a>(&'a self) -> bool {
        self.as_string().is_some()
    }

    /// If the Json value is a String, returns the associated str.
    /// Returns None otherwise.
    pub fn as_string<'a>(&'a self) -> Option<&'a str> {
        match *self {
            Json::String(ref s) => Some(&s),
            _ => None
        }
    }

    /// Returns true if the Json value is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        match *self {
            Json::I64(_) | Json::U64(_) | Json::F64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a i64. Returns false otherwise.
    pub fn is_i64(&self) -> bool {
        match *self {
            Json::I64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a u64. Returns false otherwise.
    pub fn is_u64(&self) -> bool {
        match *self {
            Json::U64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the Json value is a f64. Returns false otherwise.
    pub fn is_f64(&self) -> bool {
        match *self {
            Json::F64(_) => true,
            _ => false,
        }
    }

    /// If the Json value is a number, return or cast it to a i64.
    /// Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Json::I64(n) => Some(n),
            Json::U64(n) if n >= i64::MAX as u64 => None,
            Json::U64(n) => Some(n as i64),
            _ => None
        }
    }

    /// If the Json value is a number, return or cast it to a u64.
    /// Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Json::I64(n) if n >= 0 => Some(n as u64),
            Json::U64(n) => Some(n),
            _ => None
        }
    }

    /// If the Json value is a number, return or cast it to a f64.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Json::I64(n) => Some(n as f64),
            Json::U64(n) => Some(n as f64),
            Json::F64(n) => Some(n),
            _ => None
        }
    }

    /// Returns true if the Json value is a Boolean. Returns false otherwise.
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// If the Json value is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            &Json::Boolean(b) => Some(b),
            _ => None
        }
    }

    /// Returns true if the Json value is a Null. Returns false otherwise.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the Json value is a Null, returns ().
    /// Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match self {
            &Json::Null => Some(()),
            _ => None
        }
    }
}

impl<'a> Index<&'a str>  for Json {
    type Output = Json;

    fn index(&self, idx: &str) -> &Json {
        self.find(idx).unwrap()
    }
}

impl Index<usize> for Json {
    type Output = Json;

    fn index<'a>(&'a self, idx: usize) -> &'a Json {
        match self {
            &Json::Array(ref v) => &v[idx],
            _ => panic!("can only index Json with usize if it is an array")
        }
    }
}

/// The output of the streaming parser.
#[derive(PartialEq, Debug)]
pub enum JsonEvent {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    BooleanValue(bool),
    I64Value(i64),
    U64Value(u64),
    F64Value(f64),
    StringValue(string::String),
    NullValue,
    Error(ParserError),
}

impl FromStr for Json {
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Json, ParserError> {
        Json::from_str(s)
    }
}

impl fmt::Display for Json {
    /// Encodes a json value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shim = FormatShim { inner: f };
        let mut encoder = Encoder::new(&mut shim);
        match self.encode(&mut encoder) {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error)
        }
    }
}

impl Encodable for Json {
    fn encode<S: ::Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        match *self {
            Json::I64(v) => v.encode(e),
            Json::U64(v) => v.encode(e),
            Json::F64(v) => v.encode(e),
            Json::String(ref v) => v.encode(e),
            Json::Boolean(v) => v.encode(e),
            Json::Array(ref v) => v.encode(e),
            Json::Object(ref v) => v.encode(e),
            Json::Null => e.emit_nil(),
        }
    }
}
