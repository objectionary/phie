// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::loc::Loc;
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
        Locator::from_str($s).expect(&format!("Failed to parse locator: {}", $s))
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
        let locs_result: Result<Vec<Loc>, String> = s.split('.').map(Loc::from_str).collect();
        let p = Locator { locs: locs_result? };

        let checks: [CheckFn; 4] = [
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
                    Some(format!("{} can only be the first and only locator", p.locs[0]))
                } else {
                    None
                }
            },
        ];

        for check in checks.iter() {
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

#[test]
fn returns_none_for_out_of_bounds() {
    let locator = ph!("P.0");
    assert!(locator.loc(10).is_none());
}

#[test]
fn converts_to_vec() {
    let locator = ph!("P.0.@");
    let vec = locator.to_vec();
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[0], Loc::Pi);
}

#[test]
fn creates_from_loc() {
    let locator = Locator::from_loc(Loc::Phi);
    assert_eq!(locator.to_vec().len(), 1);
}

#[test]
fn creates_from_vec_multiple_locs() {
    let locs = vec![Loc::Pi, Loc::Attr(0), Loc::Phi];
    let locator = Locator::from_vec(locs);
    assert_eq!(locator.to_vec().len(), 3);
    assert_eq!(locator.loc(0), Some(&Loc::Pi));
    assert_eq!(locator.loc(1), Some(&Loc::Attr(0)));
    assert_eq!(locator.loc(2), Some(&Loc::Phi));
}

#[test]
fn parses_locator_with_invalid_loc() {
    let result = Locator::from_str("P.invalid.@");
    assert!(result.is_err());
}

#[test]
fn fails_on_empty_locator() {
    let result = Locator::from_str("");
    assert!(result.is_err());
}

#[test]
fn fails_on_obj_not_at_first_position() {
    let result = Locator::from_str("P.ν5");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("can only stay at the first position"));
}

#[test]
fn fails_on_root_not_at_start() {
    let result = Locator::from_str("P.Q");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("can only start a locator"));
}

#[test]
fn fails_on_attr_at_start() {
    let result = Locator::from_str("0");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("can't start a locator"));
}

#[test]
fn fails_on_obj_with_multiple_locs() {
    let result = Locator::from_str("ν5.0");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("can only be the first and only locator"));
}

#[test]
fn fails_on_trailing_dot() {
    let result = Locator::from_str("P.");
    assert!(result.is_err());
}

#[test]
fn fails_on_leading_dot() {
    let result = Locator::from_str(".P");
    assert!(result.is_err());
}
