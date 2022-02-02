// Copyright (c) 2022 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::atom::Atom;
use crate::data::Data;
use crate::path::{Item, Path};
use std::collections::HashMap;

pub struct Object {
    pub data: Option<Data>,
    pub atom: Option<Atom>,
    pub kids: HashMap<Item, Path>,
}

impl Object {
    pub fn empty() -> Object {
        Object {
            data: None,
            atom: None,
            kids: HashMap::new(),
        }
    }

    pub fn dataic(d: Data) -> Object {
        let mut obj = Object::empty();
        obj.data = Some(d);
        obj
    }

    pub fn atomic(a: Atom) -> Object {
        let mut obj = Object::empty();
        obj.atom = Some(a);
        obj
    }

    /// This object is an empty one, with nothing inside.
    pub fn is_empty(&self) -> bool {
        self.atom.is_none() && self.data.is_none() && self.kids.is_empty()
    }

    /// Add a new attribute to it, by the path item:
    ///
    /// # Examples
    ///
    /// This is how you create a new empty object and then add two
    /// attributes to it. One is `\rho`, while another one is the
    /// first child.
    ///
    /// ```
    /// use eoc::path::Item;
    /// use eoc::object::Object;
    /// use eoc::ph;
    /// let mut obj = Object::empty();
    /// obj.push(Item::Phi, ph!("v13"));
    /// obj.push(Item::Attr(0), ph!("$.1"));
    /// ```
    pub fn push(&mut self, i: Item, p: Path) -> &mut Object {
        self.kids.insert(i, p);
        self
    }

    pub fn with(&self, i: Item, p: Path) -> Object {
        let mut obj = Object::empty();
        obj.atom = self.atom.clone();
        obj.data = self.data.clone();
        obj.kids.extend(self.kids.clone().into_iter());
        obj.kids.insert(i, p);
        obj
    }
}

#[test]
fn makes_simple_object() {
    let mut obj = Object::empty();
    obj.push(Item::Attr(1), "v4".parse().unwrap());
    obj.push(Item::Rho, "$.0.@".parse().unwrap());
    assert_eq!(obj.kids.len(), 2)
}
