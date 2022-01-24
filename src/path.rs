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

use lazy_static::lazy_static;
use regex::Regex;
use rstest::rstest;
use std::fmt;
use std::str::{self, FromStr};

#[derive(Debug, Clone)]
pub struct PathError {
    msg: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Root,
    Rho,
    Phi,
    Xi,
    Sigma,
    Arg(i8),
    Obs(usize),
}

#[derive(Debug, Clone)]
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
    pub fn item(&self, id: usize) -> Option<&Item> {
        self.items.get(id)
    }
}

impl str::FromStr for Item {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE_ARG: Regex = Regex::new("^\\d+$").unwrap();
            static ref RE_OBS: Regex = Regex::new("^v\\d+$").unwrap();
        }
        if RE_ARG.is_match(s) {
            return Ok(Item::Arg(s.parse::<i8>().unwrap()));
        }
        if RE_OBS.is_match(s) {
            return Ok(Item::Obs(s[1..].parse::<usize>().unwrap()));
        }
        match s {
            "R" => Ok(Item::Root),
            "^" => Ok(Item::Rho),
            "$" => Ok(Item::Xi),
            "@" => Ok(Item::Phi),
            "&" => Ok(Item::Sigma),
            _ => Err(PathError {
                msg: format!("Unknown item '{}'", s),
            }),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = match self {
            Item::Root => "R".to_owned(),
            Item::Rho => "^".to_owned(),
            Item::Phi => "@".to_owned(),
            Item::Xi => "$".to_owned(),
            Item::Sigma => "&".to_owned(),
            Item::Arg(i) => format!("{}", i),
            Item::Obs(i) => format!("v{}", i),
        };
        f.write_str(&*s)
    }
}

type CheckFn = fn(&Path) -> Option<&Item>;
struct Check {
    check: CheckFn,
    msg: &'static str,
}

impl str::FromStr for Path {
    type Err = PathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref CHECKS: [Check; 3] = [
                Check {
                    check: |p: &Path| p.items[1..].iter().find(|i| matches!(i, Item::Obs(_))),
                    msg: "Obs can only stay at the first position"
                },
                Check {
                    check: |p: &Path| p.items[1..].iter().find(|i| matches!(i, Item::Root)),
                    msg: "Can only start a path"
                },
                Check {
                    check: |p: &Path| p.items[0..1].iter().find(|i| matches!(i, Item::Arg(_))),
                    msg: "Argument number can't start a path"
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
                return Err(PathError { msg });
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

#[rstest(
    path,
    case("R"),
    case("&"),
    case("$"),
    case("^"),
    case("@"),
    case("v78"),
    case("0"),
    case("22")
)]
fn parses_all_items(path: String) {
    assert_eq!(Item::from_str(&path).unwrap().to_string(), path)
}

#[rstest(
    path,
    case("v5.&.0.^.@.$.81"),
    case("R.0.&.3.^"),
    case("$.0"),
    case("$.0")
)]
pub fn parses_and_prints(path: String) {
    assert_eq!(ph!(&path).to_string(), path)
}

#[rstest(path,
  #[should_panic] case("v5.0.v3"),
  #[should_panic] case("R.R"),
  #[should_panic] case("5"),
  #[should_panic] case("invalid syntax"),
  #[should_panic] case("$  .  5"))]
pub fn fails_on_incorrect_path(path: String) {
    ph!(&path);
}

#[rstest]
#[case("$.0", 0, Item::Xi)]
pub fn fetches_item_from_path(#[case] path: String, #[case] idx: usize, #[case] expected: Item) {
    assert_eq!(*ph!(&path).item(idx).unwrap(), expected);
}
