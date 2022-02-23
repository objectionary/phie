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

use crate::loc::Loc;
use lazy_static::lazy_static;
use rstest::rstest;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Locator {
    locs: Vec<Loc>,
}

#[macro_export]
macro_rules! ph {
    ($s:expr) => {
        Locator::from_str($s).unwrap()
    };
}

impl Locator {
    pub fn from_vec(locs: Vec<Loc>) -> Locator {
        Locator { locs }
    }

    pub fn from_loc(loc: Loc) -> Locator {
        Locator::from_vec(vec![loc])
    }

    pub fn loc(&self, id: usize) -> Option<&Loc> {
        self.locs.get(id)
    }

    pub fn to_vec(&self) -> Vec<Loc> {
        self.locs.clone()
    }
}

type CheckFn = fn(&Locator) -> Option<&Loc>;
struct Check {
    check: CheckFn,
    msg: &'static str,
}

impl FromStr for Locator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref CHECKS: [Check; 3] = [
                Check {
                    check: |p: &Locator| p.locs[1..].iter().find(|i| matches!(i, Loc::Obj(_))),
                    msg: "ν can only stay at the first position"
                },
                Check {
                    check: |p: &Locator| p.locs[1..].iter().find(|i| matches!(i, Loc::Root)),
                    msg: "Φ can only start a locator"
                },
                Check {
                    check: |p: &Locator| p.locs[0..1].iter().find(|i| matches!(i, Loc::Attr(_))),
                    msg: "𝛼 can't start a locator"
                }
            ];
        }
        let p = Locator {
            locs: s.split('.').map(|i| Loc::from_str(i).unwrap()).collect(),
        };
        for (pos, check) in CHECKS.iter().enumerate() {
            let loc = (check.check)(&p);
            if loc.is_some() {
                let mut msg: String = String::new();
                msg.push_str(&format!("The {}th loc '{}' is wrong; ", pos, loc.unwrap()));
                msg.push_str(check.msg);
                msg.push_str(&format!("; in '{}'", s));
                return Err(msg);
            }
        }
        Ok(p)
    }
}

impl fmt::Display for Locator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(
            &self
                .locs
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
pub fn parses_and_prints(#[case] locator: String) {
    let p1 = Locator::from_str(&locator).unwrap();
    let p2 = Locator::from_str(&p1.to_string()).unwrap();
    assert_eq!(p1, p2)
}

#[test]
pub fn parses_and_prints_one() {
    let locator = "v5.&.0.^.^.@.$.81";
    let p1 = Locator::from_str(&locator).unwrap();
    let p2 = Locator::from_str(&p1.to_string()).unwrap();
    assert_eq!(p1, p2)
}

#[rstest]
#[case("v5.0.v3")]
#[case("R.R")]
#[case("5")]
#[case("invalid syntax")]
#[case("$  .  5")]
#[should_panic]
pub fn fails_on_incorrect_locator(#[case] locator: String) {
    ph!(&locator);
}

#[rstest]
#[case("$.0", 0, Loc::Xi)]
pub fn fetches_loc_from_locator(#[case] locator: String, #[case] idx: usize, #[case] expected: Loc) {
    assert_eq!(*ph!(&locator).loc(idx).unwrap(), expected);
}
