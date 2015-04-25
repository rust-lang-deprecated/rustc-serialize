/// A trait for converting values to JSON
pub trait ToJson {
    /// Converts the value of `self` to an instance of JSON
    fn to_json(&self) -> Json;
}

macro_rules! to_json_impl_i64 {
    ($($t:ty), +) => (
        $(impl ToJson for $t {
            fn to_json(&self) -> Json { Json::I64(*self as i64) }
        })+
    )
}

to_json_impl_i64! { isize, i8, i16, i32, i64 }

macro_rules! to_json_impl_u64 {
    ($($t:ty), +) => (
        $(impl ToJson for $t {
            fn to_json(&self) -> Json { Json::U64(*self as u64) }
        })+
    )
}

to_json_impl_u64! { usize, u8, u16, u32, u64 }

impl ToJson for Json {
    fn to_json(&self) -> Json { self.clone() }
}

impl ToJson for f32 {
    fn to_json(&self) -> Json { (*self as f64).to_json() }
}

impl ToJson for f64 {
    fn to_json(&self) -> Json {
        use std::num::FpCategory::{Nan, Infinite};

        match self.classify() {
            Nan | Infinite => Json::Null,
            _                  => Json::F64(*self)
        }
    }
}

impl ToJson for () {
    fn to_json(&self) -> Json { Json::Null }
}

impl ToJson for bool {
    fn to_json(&self) -> Json { Json::Boolean(*self) }
}

impl ToJson for str {
    fn to_json(&self) -> Json { Json::String(self.to_string()) }
}

impl ToJson for string::String {
    fn to_json(&self) -> Json { Json::String((*self).clone()) }
}

/// Macro rules to implement the ToJson trait for multiple tuples
macro_rules! tuple_impl {
    // use variables to indicate the arity of the tuple
    ($($tyvar:ident),* ) => {
        // the trailing commas are for the 1 tuple
        impl<
            $( $tyvar : ToJson ),*
            > ToJson for ( $( $tyvar ),* , ) {

            #[inline]
            #[allow(non_snake_case)]
            fn to_json(&self) -> Json {
                match *self {
                    ($(ref $tyvar),*,) => Json::Array(vec![$($tyvar.to_json()),*])
                }
            }
        }
    }
}

tuple_impl!{A}
tuple_impl!{A, B}
tuple_impl!{A, B, C}
tuple_impl!{A, B, C, D}
tuple_impl!{A, B, C, D, E}
tuple_impl!{A, B, C, D, E, F}
tuple_impl!{A, B, C, D, E, F, G}
tuple_impl!{A, B, C, D, E, F, G, H}
tuple_impl!{A, B, C, D, E, F, G, H, I}
tuple_impl!{A, B, C, D, E, F, G, H, I, J}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K, L}

impl<A: ToJson> ToJson for [A] {
    fn to_json(&self) -> Json { Json::Array(self.iter().map(|elt| elt.to_json()).collect()) }
}

impl<A: ToJson> ToJson for Vec<A> {
    fn to_json(&self) -> Json { Json::Array(self.iter().map(|elt| elt.to_json()).collect()) }
}

impl<A: ToJson> ToJson for BTreeMap<string::String, A> {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Json::Object(d)
    }
}

impl<A: ToJson> ToJson for HashMap<string::String, A> {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_json());
        }
        Json::Object(d)
    }
}

impl<A:ToJson> ToJson for Option<A> {
    fn to_json(&self) -> Json {
        match *self {
            None => Json::Null,
            Some(ref value) => value.to_json()
        }
    }
}
