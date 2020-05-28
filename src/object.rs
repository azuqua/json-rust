use std::{ ptr, mem, str, slice, fmt };
use std::ops::{ Index, IndexMut, Deref };

use codegen::{ DumpGenerator, Generator, PrettyGenerator };
use value::JsonValue;

use indexmap::IndexMap;
use indexmap::map::{
    Iter,
    IterMut,
    IntoIter,
    Drain
};

use std::ops::RangeFull;

static NULL: JsonValue = JsonValue::Null;

/// Helper macro for creating instances of `JsonValue::Object`.
///
/// ```
/// # #[macro_use] extern crate json;
/// # fn main() {
/// let data = object!{
///     "foo" => 42,
///     "bar" => false
/// };
///
/// assert_eq!(data["foo"], 42);
/// assert_eq!(data["bar"], false);
///
/// assert_eq!(data.dump(), r#"{"foo":42,"bar":false}"#);
/// # }
/// ```
#[macro_export]
macro_rules! object {
    // Empty object.
    {} => ($crate::JsonValue::new_object());

    // Non-empty object, no trailing comma.
    //
    // In this implementation, key/value pairs separated by commas.
    { $( $key:expr => $value:expr ),* } => {
        object!( $(
            $key => $value,
        )* )
    };

    // Non-empty object, trailing comma.
    //
    // In this implementation, the comma is part of the value.
    { $( $key:expr => $value:expr, )* } => ({
        use $crate::object::Object;

        let mut object = Object::new();

        $(
            object.insert($key, $value.into());
        )*

        $crate::JsonValue::Object(object)
    })
}

/// Helper macro for creating instances of `JsonValue::Array`.
///
/// ```
/// # #[macro_use] extern crate json;
/// # fn main() {
/// let data = array!["foo", 42, false];
///
/// assert_eq!(data[0], "foo");
/// assert_eq!(data[1], 42);
/// assert_eq!(data[2], false);
///
/// assert_eq!(data.dump(), r#"["foo",42,false]"#);
/// # }
/// ```
#[macro_export]
macro_rules! array {
    [] => ($crate::JsonValue::new_array());

    [ $( $item:expr ),* ] => ({
        let mut array = Vec::new();

        $(
            array.push($item.into());
        )*

        $crate::JsonValue::Array(array)
    })
}

/// A binary tree implementation of a string -> `JsonValue` map. You normally don't
/// have to interact with instances of `Object`, much more likely you will be
/// using the `JsonValue::Object` variant, which wraps around this struct.
#[derive(Debug, Clone)]
pub struct Object {
    inner: IndexMap<String, JsonValue>
}

impl From<IndexMap<String, JsonValue>> for Object {
    fn from(val: IndexMap<String, JsonValue>) -> Self {
        Object { inner: val }
    }
}

impl Object {
    /// Create a new, empty instance of `Object`. Empty `Object` performs no
    /// allocation until a value is inserted into it.
    #[inline(always)]
    pub fn new() -> Self {
        Object {
            inner: IndexMap::new()
        }
    }

    /// Create a new `Object` with memory preallocated for `capacity` number
    /// of entries.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Object {
            inner: IndexMap::with_capacity(capacity)
        }
    }


    /// Insert a new entry, or override an existing one. Note that `key` has
    /// to be a `&str` slice and not an owned `String`. The internals of
    /// `Object` will handle the heap allocation of the key if needed for
    /// better performance.
    #[inline]
    pub fn insert(&mut self, key: &str, value: JsonValue) {
        self.inner.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        self.inner.get_mut(key)
    }

    /// Attempts to remove the value behind `key`, if successful
    /// will return the `JsonValue` stored behind the `key`.
    pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
        self.inner.remove(key)
    }

    /// Attempts to remove the value behind `key` while preserving order, if successful
    /// will return the `JsonValue` stored behind the `key`.
    /// Computes in O(n) time (average).
    pub fn shift_remove(&mut self, key: &str) -> Option<JsonValue> {
        println!("inner shift remove called");
        dbg!();
        self.inner.shift_remove(key)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Wipe the `Object` clear. The capacity will remain untouched.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter<String, JsonValue> {
        self.inner.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut<String, JsonValue> {
        self.inner.iter_mut()
    }

    pub fn drain(&mut self, range: RangeFull) -> Drain<String, JsonValue> {
        self.inner.drain(range)
    }

    pub fn into_iter(self) -> IntoIter<String, JsonValue> {
        self.inner.into_iter()
    }

    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_object(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_object(self).expect("Can't fail");
        gen.consume()
    }
}

// Because keys can inserted in different order, the safe way to
// compare `Object`s is to iterate over one and check if the other
// has all the same keys.
impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, value) in self.iter() {
            match other.get(key) {
                Some(ref other_val) => if *other_val != value { return false; },
                None                => return false
            }
        }

        true
    }
}

impl Eq for Object {}

/// Implements indexing by `&str` to easily access object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// # use json::JsonValue;
/// #
/// # fn main() {
/// let value = object!{
///     "foo" => "bar"
/// };
///
/// if let JsonValue::Object(object) = value {
///   assert!(object["foo"] == "bar");
/// }
/// # }
/// ```
// TODO: doc
impl<'a> Index<&'a str> for Object {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &JsonValue {
        match self.get(index) {
            Some(value) => value,
            _ => &NULL
        }
    }
}

impl Index<String> for Object {
    type Output = JsonValue;

    fn index(&self, index: String) -> &JsonValue {
        self.index(index.deref())
    }
}

impl<'a> Index<&'a String> for Object {
    type Output = JsonValue;

    fn index(&self, index: &String) -> &JsonValue {
        self.index(index.deref())
    }
}

/// Implements mutable indexing by `&str` to easily modify object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// # use json::JsonValue;
/// #
/// # fn main() {
/// let value = object!{};
///
/// if let JsonValue::Object(mut object) = value {
///   object["foo"] = 42.into();
///
///   assert!(object["foo"] == 42);
/// }
/// # }
/// ```
impl<'a> IndexMut<&'a str> for Object {
    fn index_mut(&mut self, index: &str) -> &mut JsonValue {
        if self.get(index).is_none() {
            self.insert(index, JsonValue::Null);
        }
        self.get_mut(index).unwrap()
    }
}

impl IndexMut<String> for Object {
    fn index_mut(&mut self, index: String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}

impl<'a> IndexMut<&'a String> for Object {
    fn index_mut(&mut self, index: &String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}
