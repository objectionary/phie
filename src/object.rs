// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt, str::FromStr};

use itertools::Itertools;
use regex::Regex;
use rstest::rstest;

use crate::{atom::*, data::Data, loc::Loc, locator::Locator};

pub type Ob = usize;

pub struct Object {
    pub delta:    Option<Data>,
    pub lambda:   Option<(String, Atom)>,
    pub constant: bool,
    pub attrs:    HashMap<Loc, (Locator, bool)>
}

impl Object {
    pub fn open() -> Object {
        Object {
            delta:    None,
            lambda:   None,
            constant: false,
            attrs:    HashMap::new()
        }
    }

    pub fn dataic(d: Data) -> Object {
        Object {
            delta:    Some(d),
            lambda:   None,
            constant: true,
            attrs:    HashMap::new()
        }
    }

    pub fn atomic(n: String, a: Atom) -> Object {
        Object {
            delta:    None,
            lambda:   Some((n, a)),
            constant: false,
            attrs:    HashMap::new()
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
    /// use std::str::FromStr;
    ///
    /// use phie::{loc::Loc, locator::Locator, object::Object, ph};
    /// let mut obj = Object::open();
    /// obj.push(Loc::Phi, ph!("ν13"), false);
    /// obj.push(Loc::Attr(0), ph!("ρ.1"), false);
    /// ```
    pub fn push(&mut self, loc: Loc, p: Locator, xi: bool) -> &mut Object {
        self.attrs.insert(loc, (p, xi));
        self
    }

    /// You can do the same, but with "fluent interface" of the `Object`.
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// use phie::{loc::Loc, locator::Locator, object::Object, ph};
    /// let obj =
    ///     Object::open()
    ///         .with(Loc::Phi, ph!("ν13"), false)
    ///         .with(Loc::Attr(0), ph!("ρ.1"), false);
    /// ```
    pub fn with(&self, loc: Loc, p: Locator, xi: bool) -> Object {
        let mut obj = self.copy();
        obj.attrs.insert(loc, (p, xi));
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
        obj.delta = self.delta;
        obj.attrs.extend(self.attrs.clone());
        obj
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = vec![];
        if let Some(a) = &self.lambda {
            parts.push(format!("λ↦{}", a.0));
        }
        if let Some(p) = &self.delta {
            parts.push(format!("Δ↦0x{:04X}", p));
        }
        for i in self.attrs.iter() {
            let (attr, (locator, xi)) = i;
            let suffix = if *xi {
                "(ξ)".to_string()
            } else if locator.loc(0).is_some_and(|loc| matches!(loc, Loc::Obj(_))) {
                "(𝜋)".to_string()
            } else {
                "".to_string()
            };
            parts.push(format!("{}↦{}", attr, locator) + &suffix);
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
        let re = Regex::new("⟦(!?)(.*)⟧")
            .map_err(|e| format!("Invalid object regex pattern: {}", e))?;
        let mut obj = Object::open();
        let caps = re
            .captures(s)
            .ok_or_else(|| format!("Can't parse object format in '{}'", s))?;
        let inner = caps
            .get(2)
            .ok_or_else(|| format!("Missing object body in '{}'", s))?
            .as_str()
            .trim();
        for pair in inner.split(',').map(|t| t.trim()) {
            let (i, p) = pair
                .split('↦')
                .map(|t| t.trim())
                .collect_tuple()
                .ok_or_else(|| format!("Can't split '{}' in two parts at '{}'", pair, s))?;
            let first_char = i
                .chars()
                .next()
                .ok_or_else(|| format!("Empty attribute name in '{}'", pair))?;
            match first_char {
                'λ' => {
                    let lambda_fn = match p {
                        "int-times" => int_times,
                        "int-div" => int_div,
                        "int-sub" => int_sub,
                        "int-add" => int_add,
                        "int-neg" => int_neg,
                        "bool-if" => bool_if,
                        "int-less" => int_less,
                        _ => return Err(format!("Unknown lambda '{}' in '{}'", p, s))
                    };
                    obj = Object::atomic(p.to_string(), lambda_fn);
                }
                'Δ' => {
                    let hex: String = p.chars().skip(2).collect();
                    let data = Data::from_str_radix(&hex, 16)
                        .map_err(|e| format!("Can't parse hex '{}' in '{}': {}", hex, s, e))?;
                    obj = Object::dataic(data);
                }
                _ => {
                    let tail = if p.ends_with("(𝜋)") {
                        p.chars().take(p.len() - "(𝜋)".len() - 1).collect()
                    } else {
                        p.to_string()
                    };
                    let xi_suffix = "(ξ)";
                    let xi = tail.ends_with(xi_suffix);
                    let locator = if xi {
                        tail.chars()
                            .take(tail.len() - xi_suffix.len() - 1)
                            .collect()
                    } else {
                        tail.to_string()
                    };
                    let loc = Loc::from_str(i)
                        .map_err(|e| format!("Can't parse location '{}': {}", i, e))?;
                    let locator_parsed = Locator::from_str(&locator)
                        .map_err(|e| format!("Can't parse locator '{}': {}", locator, e))?;
                    obj.push(loc, locator_parsed, xi);
                }
            };
        }
        let constant_flag = caps
            .get(1)
            .ok_or_else(|| format!("Missing constant flag capture in '{}'", s))?
            .as_str();
        if !constant_flag.is_empty() {
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
    obj.push(Loc::Attr(1), "ν4".parse().unwrap(), false);
    obj.push(Loc::Rho, "P.0.@".parse().unwrap(), false);
    assert_eq!(obj.attrs.len(), 2)
}

#[test]
fn extends_by_making_new_object() {
    let obj = Object::open()
        .with(Loc::Attr(1), ph!("ν14"), false)
        .with(Loc::Phi, ph!("^.@"), false)
        .with(Loc::Rho, ph!("P.^.0.0.^.@"), false);
    assert_eq!(obj.attrs.len(), 3);
    assert!(obj.delta.is_none());
    assert!(obj.lambda.is_none());
}

#[test]
fn prints_and_parses_simple_object() {
    let mut obj = Object::open();
    obj.constant = true;
    obj.push(Loc::Attr(1), "ν4".parse().unwrap(), false);
    obj.push(Loc::Rho, "P.0.@".parse().unwrap(), false);
    let text = obj.to_string();
    assert_eq!("⟦! ρ↦𝜋.𝛼0.𝜑, 𝛼1↦ν4(𝜋)⟧", text);
    let obj2 = Object::from_str(&text).unwrap();
    assert_eq!(obj2.to_string(), text);
}

#[rstest]
#[case("ν7(𝜋) ↦ ⟦! λ ↦ int-sub, ρ ↦ 𝜋.𝜋.𝛼0, 𝛼0 ↦ ν8(𝜋) ⟧")]
#[case("ν7(𝜋) ↦ ⟦ Δ ↦ 0x0001 ⟧")]
#[case("ν11(𝜋) ↦ ⟦ λ ↦ int-add, ρ ↦ ν9(𝜋), 𝛼0 ↦ ν10(𝜋) ⟧")]
fn prints_and_parses_some_object(#[case] text: String) {
    let obj1 = Object::from_str(&text).unwrap();
    let text2 = obj1.to_string();
    let obj2 = Object::from_str(&text2).unwrap();
    let text3 = obj2.to_string();
    assert_eq!(text2, text3);
}

#[test]
fn fails_on_unknown_lambda() {
    let text = "⟦ λ ↦ unknown-lambda ⟧";
    let result = Object::from_str(text);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.contains("Unknown lambda"),
        "Expected 'Unknown lambda' but got: {}",
        err
    );
}

#[test]
fn fails_on_invalid_format() {
    let text = "invalid object format";
    let result = Object::from_str(text);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse object format"));
}

#[test]
fn fails_on_invalid_hex() {
    let text = "⟦ Δ ↦ 0xZZZZ ⟧";
    let result = Object::from_str(text);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse hex"));
}

#[test]
fn fails_on_malformed_attribute() {
    let text = "⟦ malformed ⟧";
    let result = Object::from_str(text);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't split"));
}

#[test]
fn parses_object_with_xi() {
    let text = "⟦ 𝜑 ↦ ν2(ξ) ⟧";
    let obj = Object::from_str(text).unwrap();
    assert_eq!(obj.attrs.len(), 1);
    let (_, xi) = obj.attrs.get(&Loc::Phi).unwrap();
    assert!(*xi);
}

#[test]
fn parses_object_without_xi() {
    let text = "⟦ ρ ↦ 𝜋 ⟧";
    let obj = Object::from_str(text).unwrap();
    assert_eq!(obj.attrs.len(), 1);
    let (_, xi) = obj.attrs.get(&Loc::Rho).unwrap();
    assert!(!*xi);
}

#[test]
fn fails_on_empty_attribute_name() {
    let text = "⟦ ↦ ν0 ⟧";
    let result = Object::from_str(text);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Empty attribute name"));
}

#[test]
fn fails_on_invalid_loc_in_attribute() {
    let text = "⟦ invalid_loc ↦ ν0 ⟧";
    let result = Object::from_str(text);
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse location"));
}
