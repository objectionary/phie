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

use crate::atom::*;
use crate::data::Data;
use crate::loc::Loc;
use crate::locator::Locator;
use itertools::Itertools;
use regex::Regex;
use rstest::rstest;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub type Ob = usize;

pub struct Object {
    pub delta: Option<Data>,
    pub lambda: Option<Atom>,
    pub constant: bool,
    pub attrs: HashMap<Loc, (Locator, bool)>,
}

impl Object {
    pub fn open() -> Object {
        Object {
            delta: None,
            lambda: None,
            constant: false,
            attrs: HashMap::new(),
        }
    }

    pub fn dataic(d: Data) -> Object {
        Object {
            delta: Some(d),
            lambda: None,
            constant: true,
            attrs: HashMap::new(),
        }
    }

    pub fn atomic(a: Atom) -> Object {
        Object {
            delta: None,
            lambda: Some(a),
            constant: false,
            attrs: HashMap::new(),
        }
    }

    /// This object is an empty one, with nothing inside.
    pub fn is_empty(&self) -> bool {
        self.lambda.is_none() && self.delta.is_none() && self.attrs.is_empty()
    }

    /// Add a new attribute to it, by the locator loc:
    ///
    /// # Examples
    ///
    /// This is how you create a new empty object and then add two
    /// attributes to it. One is `\rho`, while another one is the
    /// first child.
    ///
    /// ```
    /// use eoc::loc::Loc;
    /// use eoc::locator::Locator;
    /// use eoc::object::Object;
    /// use std::str::FromStr;
    /// use eoc::ph;
    /// let mut obj = Object::open();
    /// obj.push(Loc::Phi, ph!("v13"), false);
    /// obj.push(Loc::Attr(0), ph!("$.1"), false);
    /// ```
    ///
    pub fn push(&mut self, loc: Loc, p: Locator, psi: bool) -> &mut Object {
        self.attrs.insert(loc, (p, psi));
        self
    }

    /// You can do the same, but with "fluent interface" of the `Object`.
    ///
    /// ```
    /// use eoc::loc::Loc;
    /// use eoc::locator::Locator;
    /// use eoc::object::Object;
    /// use std::str::FromStr;
    /// use eoc::ph;
    /// let obj = Object::open()
    ///   .with(Loc::Phi, ph!("v13"), false)
    ///   .with(Loc::Attr(0), ph!("$.1"), false);
    /// ```
    pub fn with(&self, loc: Loc, p: Locator, psi: bool) -> Object {
        let mut obj = self.copy();
        obj.attrs.insert(loc, (p, psi));
        obj
    }

    pub fn as_constant(&self) -> Object {
        let mut obj = self.copy();
        obj.constant = true;
        obj
    }

    fn copy(&self) -> Object {
        let mut obj = Object::open();
        obj.lambda = self.lambda.clone();
        obj.constant = self.constant;
        obj.delta = self.delta.clone();
        obj.attrs.extend(self.attrs.clone().into_iter());
        obj
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = vec![];
        if let Some(_) = self.lambda {
            parts.push("λ↦noop".to_string());
        }
        if let Some(p) = self.delta {
            parts.push(format!("Δ↦0x{:04X}", p));
        }
        for i in self.attrs.iter() {
            let (attr, (locator, psi)) = i;
            parts.push(
                format!("{}↦{}", attr, locator)
                    + &(if *psi {
                        "(𝜓)".to_string()
                    } else {
                        "".to_string()
                    }),
            );
        }
        parts.sort();
        write!(
            f,
            "⟦{}{}⟧",
            if self.constant { "! " } else { "" },
            parts.iter().join(", ")
        )
    }
}

impl FromStr for Object {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new("⟦(!?)(.*)⟧").unwrap();
        let mut obj = Object::open();
        let caps = re.captures(s).unwrap();
        for pair in caps
            .get(2)
            .unwrap()
            .as_str()
            .trim()
            .split(",")
            .map(|t| t.trim())
        {
            let (i, p) = pair
                .split("↦")
                .map(|t| t.trim())
                .collect_tuple()
                .ok_or(format!("Can't split '{}' in two parts at '{}'", pair, s))?;
            match i.chars().take(1).last().unwrap() {
                'λ' => {
                    obj = Object::atomic(match p {
                        "noop" => noop,
                        "int-sub" => int_sub,
                        "int-add" => int_add,
                        "bool-if" => bool_if,
                        "int-less" => int_less,
                        _ => panic!("Unknown lambda '{}'", p),
                    });
                }
                'Δ' => {
                    let hex: String = p.chars().skip(2).collect();
                    let data: Data = Data::from_str_radix(&hex, 16)
                        .expect(&format!("Can't parse hex '{}' in '{}'", hex, s));
                    obj = Object::dataic(data);
                }
                _ => {
                    let psi_suffix = "(𝜓)";
                    let psi = p.ends_with(psi_suffix);
                    let locator = if psi {
                        p.chars().take(p.len() - psi_suffix.len() - 1).collect()
                    } else {
                        p.to_string()
                    };
                    obj.push(
                        Loc::from_str(i).unwrap(),
                        Locator::from_str(&locator).unwrap(),
                        psi,
                    );
                }
            };
        }
        if !caps.get(1).unwrap().as_str().is_empty() {
            obj.constant = true;
        }
        Ok(obj)
    }
}

#[cfg(test)]
use crate::ph;

#[test]
fn makes_simple_object() {
    let mut obj = Object::open();
    obj.push(Loc::Attr(1), "v4".parse().unwrap(), false);
    obj.push(Loc::Rho, "$.0.@".parse().unwrap(), false);
    assert_eq!(obj.attrs.len(), 2)
}

#[test]
fn extends_by_making_new_object() {
    let obj = Object::open()
        .with(Loc::Attr(1), ph!("v14.^"), false)
        .with(Loc::Phi, ph!("v7.@"), false)
        .with(Loc::Rho, ph!("$.^.0.0.^.@"), false);
    assert_eq!(obj.attrs.len(), 3);
    assert!(obj.delta.is_none());
    assert!(obj.lambda.is_none());
}

#[test]
fn prints_and_parses_simple_object() {
    let mut obj = Object::open();
    obj.constant = true;
    obj.push(Loc::Attr(1), "v4".parse().unwrap(), false);
    obj.push(Loc::Rho, "$.0.@".parse().unwrap(), false);
    let text = obj.to_string();
    assert_eq!("⟦! ρ↦ξ.𝛼0.φ, 𝛼1↦ν4⟧", text);
    let obj2 = Object::from_str(&text).unwrap();
    assert_eq!(obj2.to_string(), text);
}

#[rstest]
#[case("ν7 ↦ ⟦! λ ↦ int-sub, ρ ↦ 𝜓.𝜓.𝛼0, 𝛼0 ↦ ν8 ⟧")]
#[case("ν7 ↦ ⟦ Δ ↦ 0x0001 ⟧")]
#[case("ν11 ↦ ⟦ λ ↦ int-add, ρ ↦ ν9, 𝛼0 ↦ ν10 ⟧")]
fn prints_and_parses_some_object(#[case] text: String) {
    let obj1 = Object::from_str(&text).unwrap();
    let text2 = obj1.to_string();
    let obj2 = Object::from_str(&text2).unwrap();
    let text3 = obj2.to_string();
    assert_eq!(text2, text3);
}
