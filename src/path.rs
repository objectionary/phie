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

use crate::object::Ob;
use lazy_static::lazy_static;
use regex::Regex;
use rstest::rstest;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Item {
    Root,
    Rho,
    Phi,
    Xi,
    Sigma,
    Attr(i8),
    Obj(Ob),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    items: Vec<Item>,
}

#[macro_export]
macro_rules! ph {
    ($s:expr) => {
        Path::from_str($s).unwrap()
    };
}

impl Path {
    pub fn from_vec(items : Vec<Item>) -> Path {
        Path { items }
    }

    pub fn from_item(item : Item) -> Path {
        Path::from_vec(vec![item])
    }

    pub fn item(&self, id: usize) -> Option<&Item> {
        self.items.get(id)
    }

    pub fn to_vec(&self) -> Vec<Item> {
        self.items.clone()
    }
}

impl FromStr for Item {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE_ARG: Regex = Regex::new("^𝛼?(\\d+)$").unwrap();
            static ref RE_OBS: Regex = Regex::new("^[v|ν](\\d+)$").unwrap();
        }
        if let Some(caps) = RE_ARG.captures(s) {
            return Ok(Item::Attr(caps.get(1).unwrap().as_str().parse::<i8>().unwrap()));
        }
        if let Some(caps) = RE_OBS.captures(s) {
            return Ok(Item::Obj(caps.get(1).unwrap().as_str().parse::<usize>().unwrap()));
        }
        match s {
            "R" => Ok(Item::Root),
            "Φ" => Ok(Item::Root),
            "^" => Ok(Item::Rho),
            "ρ" => Ok(Item::Rho),
            "$" => Ok(Item::Xi),
            "ξ" => Ok(Item::Xi),
            "@" => Ok(Item::Phi),
            "φ" => Ok(Item::Phi),
            "&" => Ok(Item::Sigma),
            "σ" => Ok(Item::Sigma),
            _ => Err(format!("Unknown item '{}'", s)),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = match self {
            Item::Root => "Φ".to_owned(),
            Item::Rho => "ρ".to_owned(),
            Item::Phi => "φ".to_owned(),
            Item::Xi => "ξ".to_owned(),
            Item::Sigma => "σ".to_owned(),
            Item::Attr(i) => format!("𝛼{}", i),
            Item::Obj(i) => format!("ν{}", i),
        };
        f.write_str(&*s)
    }
}

type CheckFn = fn(&Path) -> Option<&Item>;
struct Check {
    check: CheckFn,
    msg: &'static str,
}

impl FromStr for Path {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref CHECKS: [Check; 3] = [
                Check {
                    check: |p: &Path| p.items[1..].iter().find(|i| matches!(i, Item::Obj(_))),
                    msg: "ν can only stay at the first position"
                },
                Check {
                    check: |p: &Path| p.items[1..].iter().find(|i| matches!(i, Item::Root)),
                    msg: "Φ can only start a path"
                },
                Check {
                    check: |p: &Path| p.items[0..1].iter().find(|i| matches!(i, Item::Attr(_))),
                    msg: "𝛼 can't start a path"
                }
            ];
        }
        let p = Path {
            items: s.split('.').map(|i| Item::from_str(i).unwrap()).collect(),
        };
        for (pos, check) in CHECKS.iter().enumerate() {
            let item = (check.check)(&p);
            if item.is_some() {
                let mut msg: String = String::new();
                msg.push_str(&format!(
                    "The {}th item '{}' is wrong; ",
                    pos,
                    item.unwrap()
                ));
                msg.push_str(check.msg);
                msg.push_str(&format!("; in '{}'", s));
                return Err(msg);
            }
        }
        Ok(p)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            &self
                .items
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join("."),
        )
    }
}

#[rstest]
#[case("R")]
#[case("&")]
#[case("$")]
#[case("^")]
#[case("@")]
#[case("v78")]
#[case("v5.&.0.^.@.$.81")]
#[case("R.0.&.3.^")]
#[case("Φ.𝛼0.σ.𝛼3.ρ")]
#[case("$.0")]
#[case("$.0")]
pub fn parses_and_prints(#[case] path: String) {
    let p1 = Path::from_str(&path).unwrap();
    let p2 = Path::from_str(&p1.to_string()).unwrap();
    assert_eq!(p1, p2)
}

#[test]
pub fn parses_and_prints_one() {
    let path = "v5.&.0.^.^.@.$.81";
    let p1 = Path::from_str(&path).unwrap();
    let p2 = Path::from_str(&p1.to_string()).unwrap();
    assert_eq!(p1, p2)
}

#[rstest]
#[case("v5.0.v3")]
#[case("R.R")]
#[case("5")]
#[case("invalid syntax")]
#[case("$  .  5")]
#[should_panic]
pub fn fails_on_incorrect_path(#[case] path: String) {
    ph!(&path);
}

#[rstest]
#[case("$.0", 0, Item::Xi)]
pub fn fetches_item_from_path(#[case] path: String, #[case] idx: usize, #[case] expected: Item) {
    assert_eq!(*ph!(&path).item(idx).unwrap(), expected);
}
