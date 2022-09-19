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

/// Locator is a chain of attributes connected with dots,
/// for example `𝜋.𝜋.𝛼0` is a locator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locator {
    locs: Vec<Loc>,
}

/// Use this macro to create a locator faster:
///
/// ```
/// use phie::ph;
/// use phie::loc::Loc;
/// use phie::locator::Locator;
/// use std::str::FromStr;
/// let k = ph!("𝜋.𝜋.𝛼0");
/// ```
#[macro_export]
macro_rules! ph {
    ($s:expr) => {
        Locator::from_str($s).unwrap()
    };
}

impl Locator {
    /// Make a locator from a vector of attribute names:
    ///
    /// ```
    /// use phie::loc::Loc;
    /// use phie::locator::Locator;
    /// let k = Locator::from_vec(vec![Loc::Phi, Loc::Delta]);
    /// ```
    pub fn from_vec(locs: Vec<Loc>) -> Locator {
        Locator { locs }
    }

    /// Make a locator from a single attribute:
    ///
    /// ```
    /// use phie::loc::Loc;
    /// use phie::locator::Locator;
    /// let k = Locator::from_loc(Loc::Phi);
    /// ```
    pub fn from_loc(loc: Loc) -> Locator {
        Locator::from_vec(vec![loc])
    }

    /// Get a single attribute from the locator, by its position.
    pub fn loc(&self, id: usize) -> Option<&Loc> {
        self.locs.get(id)
    }

    /// Turn it into a vector of attributes.
    pub fn to_vec(&self) -> Vec<Loc> {
        self.locs.clone()
    }
}

type CheckFn = fn(&Locator) -> Option<String>;

impl FromStr for Locator {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref CHECKS: [CheckFn; 4] = [
                |p: &Locator| -> Option<String> {
                    p.locs[1..]
                        .iter()
                        .find(|i| matches!(i, Loc::Obj(_)))
                        .map(|v| format!("{} can only stay at the first position", v))
                },
                |p: &Locator| {
                    p.locs[1..]
                        .iter()
                        .find(|i| matches!(i, Loc::Root))
                        .map(|v| format!("{} can only start a locator", v))
                },
                |p: &Locator| {
                    p.locs[0..1]
                        .iter()
                        .find(|i| matches!(i, Loc::Attr(_)))
                        .map(|v| format!("{} can't start a locator", v))
                },
                |p: &Locator| {
                    if matches!(p.locs[0], Loc::Obj(_)) && p.locs.len() > 1 {
                        Some(format!(
                            "{} can only be the first and only locator",
                            p.locs[0]
                        ))
                    } else {
                        None
                    }
                },
            ];
        }
        let p = Locator {
            locs: s.split('.').map(|i| Loc::from_str(i).unwrap()).collect(),
        };
        for check in CHECKS.iter() {
            if let Some(msg) = (check)(&p) {
                return Err(format!("{} in '{}'", msg, p));
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
#[case("Q")]
#[case("&")]
#[case("P")]
#[case("^")]
#[case("@")]
#[case("ν78")]
#[case("ρ.&.0.^.@.P.81")]
#[case("Q.0.&.3.^")]
#[case("𝜑.𝛼0.σ.𝛼3.ρ")]
#[case("Φ.𝛼1")]
#[case("𝜋.𝜋.𝛼9")]
#[case("P.0")]
#[case("P.0")]
pub fn parses_and_prints(#[case] locator: String) {
    let p1 = Locator::from_str(&locator).unwrap();
    let p2 = Locator::from_str(&p1.to_string()).unwrap();
    assert_eq!(p1, p2)
}

#[test]
pub fn parses_and_prints_one() {
    let locator = "ρ.&.0.^.^.@.P.81";
    let p1 = Locator::from_str(locator).unwrap();
    let p2 = Locator::from_str(&p1.to_string()).unwrap();
    assert_eq!(p1, p2)
}

#[rstest]
#[case("")]
#[case("ν5.0.ν3")]
#[case("𝜋.")]
#[case(".ν5")]
#[case("𝜋.ν5")]
#[case("Q.Q")]
#[case("5")]
#[case("invalid syntax")]
#[case("$  .  5")]
#[should_panic]
pub fn fails_on_incorrect_locator(#[case] locator: String) {
    ph!(&locator);
}

#[rstest]
#[case("P.0", 0, Loc::Pi)]
pub fn fetches_loc_from_locator(
    #[case] locator: String,
    #[case] idx: usize,
    #[case] expected: Loc,
) {
    assert_eq!(*ph!(&locator).loc(idx).unwrap(), expected);
}
