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
use crate::ph;
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt;
use itertools::Itertools;

pub type Ob = usize;

pub struct Object {
    pub psi: Option<usize>,
    pub delta: Option<Data>,
    pub lambda: Option<Atom>,
    pub attrs: HashMap<Item, Path>,
}

impl Object {
    pub fn open() -> Object {
        Object {
            psi: None,
            delta: None,
            lambda: None,
            attrs: HashMap::new(),
        }
    }

    pub fn copy(ob: usize) -> Object {
        Object {
            psi: Some(ob),
            delta: None,
            lambda: None,
            attrs: HashMap::new(),
        }
    }

    pub fn dataic(d: Data) -> Object {
        Object {
            psi: None,
            delta: Some(d),
            lambda: None,
            attrs: HashMap::new(),
        }
    }

    pub fn atomic(a: Atom) -> Object {
        Object {
            psi: None,
            delta: None,
            lambda: Some(a),
            attrs: HashMap::new(),
        }
    }

    /// This object is an empty one, with nothing inside.
    pub fn is_empty(&self) -> bool {
        self.psi.is_none() && self.lambda.is_none() &&
            self.delta.is_none() && self.attrs.is_empty()
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
    /// let mut obj = Object::open();
    /// obj.push(Item::Phi, ph!("v13"));
    /// obj.push(Item::Attr(0), ph!("$.1"));
    /// ```
    ///
    /// You can do the same, but with "fluent interface" of the `Object`.
    ///
    /// ```
    /// use eoc::path::Item;
    /// use eoc::object::Object;
    /// use eoc::ph;
    /// let obj = Object::open()
    ///   .with(Item::Phi, ph!("v13"))
    ///   .with(Item::Attr(0), ph!("$.1"));
    /// ```
    pub fn push(&mut self, i: Item, p: Path) -> &mut Object {
        self.attrs.insert(i, p);
        self
    }

    pub fn with(&self, i: Item, p: Path) -> Object {
        let mut obj = Object::open();
        obj.psi = self.psi.clone();
        obj.lambda = self.lambda.clone();
        obj.delta = self.delta.clone();
        obj.attrs.extend(self.attrs.clone().into_iter());
        obj.attrs.insert(i, p);
        obj
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = vec![];
        if let Some(p) = self.psi {
            parts.push(format!("ψ:ν{}", p));
        }
        if let Some(_) = self.lambda {
            parts.push("λ".to_string());
        }
        if let Some(p) = self.delta {
            parts.push(format!("Δ:0x{:04X}", p));
        }
        for i in self.attrs.iter() {
            let (attr, path) = i;
            parts.push(
                match attr {
                    Item::Rho => "ρ".to_string(),
                    Item::Phi => "φ".to_string(),
                    _ => format!("𝛼{}", attr)
                } + &format!(":{}", path)
            );
        }
        parts.sort();
        write!(f, "{}", parts.iter().join(" "))
    }
}

#[test]
fn makes_simple_object() {
    let mut obj = Object::open();
    obj.push(Item::Attr(1), "v4".parse().unwrap());
    obj.push(Item::Rho, "$.0.@".parse().unwrap());
    assert_eq!(obj.attrs.len(), 2)
}

#[test]
fn extends_by_making_new_object() {
    let obj = Object::open()
        .with(Item::Attr(1), ph!("v14.^"))
        .with(Item::Phi, ph!("v7.@"))
        .with(Item::Rho, ph!("$.^.0.0.^.@"));
    assert_eq!(obj.attrs.len(), 3);
    assert!(obj.delta.is_none());
    assert!(obj.lambda.is_none());
}

#[test]
fn prints_simple_object() {
    let mut obj = Object::open();
    obj.push(Item::Attr(1), "v4".parse().unwrap());
    obj.push(Item::Rho, "$.0.@".parse().unwrap());
    assert_eq!("ρ:$.0.@ 𝛼1:v4", obj.to_string())
}

