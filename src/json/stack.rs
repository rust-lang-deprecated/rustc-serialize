use std::{str, string};

/// A Stack represents the current position of the parser in the logical
/// structure of the JSON stream.
/// For example foo.bar[3].x
pub struct Stack {
    stack: Vec<InternalStackElement>,
    str_buffer: Vec<u8>,
}

/// StackElements compose a Stack.
/// For example, Key("foo"), Key("bar"), Index(3) and Key("x") are the
/// StackElements compositing the stack that represents foo.bar[3].x
#[derive(PartialEq, Clone, Debug)]
pub enum StackElement<'l> {
    Index(u32),
    Key(&'l str),
}

// Internally, Key elements are stored as indices in a buffer to avoid
// allocating a string for every member of an object.
#[derive(PartialEq, Clone, Debug)]
enum InternalStackElement {
    InternalIndex(u32),
    InternalKey(u16, u16), // start, size
}

impl Stack {
    pub fn new() -> Stack {
        Stack { stack: Vec::new(), str_buffer: Vec::new() }
    }

    /// Returns The number of elements in the Stack.
    pub fn len(&self) -> usize { self.stack.len() }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool { self.stack.is_empty() }

    /// Provides access to the StackElement at a given index.
    /// lower indices are at the bottom of the stack while higher indices are
    /// at the top.
    pub fn get<'l>(&'l self, idx: usize) -> StackElement<'l> {
        match self.stack[idx] {
            InternalStackElement::InternalIndex(i) => StackElement::Index(i),
            InternalStackElement::InternalKey(start, size) => {
                StackElement::Key(str::from_utf8(
                    &self.str_buffer[start as usize .. start as usize + size as usize]).unwrap())
            }
        }
    }

    /// Compares this stack with an array of StackElements.
    pub fn is_equal_to(&self, rhs: &[StackElement]) -> bool {
        if self.stack.len() != rhs.len() { return false; }
        for i in 0..rhs.len() {
            if self.get(i) != rhs[i] { return false; }
        }
        return true;
    }

    /// Returns true if the bottom-most elements of this stack are the same as
    /// the ones passed as parameter.
    pub fn starts_with(&self, rhs: &[StackElement]) -> bool {
        if self.stack.len() < rhs.len() { return false; }
        for i in 0..rhs.len() {
            if self.get(i) != rhs[i] { return false; }
        }
        return true;
    }

    /// Returns true if the top-most elements of this stack are the same as
    /// the ones passed as parameter.
    pub fn ends_with(&self, rhs: &[StackElement]) -> bool {
        if self.stack.len() < rhs.len() { return false; }
        let offset = self.stack.len() - rhs.len();
        for i in 0..rhs.len() {
            if self.get(i + offset) != rhs[i] { return false; }
        }
        return true;
    }

    /// Returns the top-most element (if any).
    pub fn top<'l>(&'l self) -> Option<StackElement<'l>> {
        return match self.stack.last() {
            None => None,
            Some(&InternalStackElement::InternalIndex(i)) => Some(StackElement::Index(i)),
            Some(&InternalStackElement::InternalKey(start, size)) => {
                Some(StackElement::Key(str::from_utf8(
                    &self.str_buffer[start as usize .. (start+size) as usize]
                ).unwrap()))
            }
        }
    }
}

// All the method of this trait are part of the Stack structure, 
// but have to be exposed to other modules but not to the external 
// users of the library. That'a why those methods are exposed via a trait.
#[doc(hidden)]
pub trait SemiPubMethods{
    fn push_key(&mut self, key: string::String);
    fn push_index(&mut self, index: u32);
    fn pop(&mut self);
    fn last_is_index(&self) -> bool;
    fn bump_index(&mut self);
}

impl SemiPubMethods for Stack {
    fn push_key(&mut self, key: string::String) {
        self.stack.push(InternalStackElement::InternalKey(
                self.str_buffer.len() as u16,
                key.len() as u16));
        for c in key.as_bytes().iter() {
            self.str_buffer.push(*c);
        }
    }

    // Used by Parser to insert Index elements at the top of the stack.
    fn push_index(&mut self, index: u32) {
        self.stack.push(InternalStackElement::InternalIndex(index));
    }

    // Used by Parser to remove the top-most element of the stack.
    fn pop(&mut self) {
        assert!(!self.is_empty());
        match *self.stack.last().unwrap() {
            InternalStackElement::InternalKey(_, sz) => {
                let new_size = self.str_buffer.len() - sz as usize;
                self.str_buffer.truncate(new_size);
            }
            InternalStackElement::InternalIndex(_) => {}
        }
        self.stack.pop();
    }

    // Used by Parser to test whether the top-most element is an index.
    fn last_is_index(&self) -> bool {
        if self.is_empty() { return false; }
        return match *self.stack.last().unwrap() {
            InternalStackElement::InternalIndex(_) => true,
            _ => false,
        }
    }

    // Used by Parser to increment the index of the top-most element.
    fn bump_index(&mut self) {
        let len = self.stack.len();
        let idx = match *self.stack.last().unwrap() {
            InternalStackElement::InternalIndex(i) => { i + 1 }
            _ => { panic!(); }
        };
        self.stack[len - 1] = InternalStackElement::InternalIndex(idx);
    }
}
