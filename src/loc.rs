// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::object::Ob;
use lazy_static::lazy_static;
use regex::Regex;
use rstest::rstest;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Loc {
    Root,
    Rho,
    Phi,
    Pi,
    Delta,
    Sigma,
    Attr(i8),
    Obj(Ob),
}

impl FromStr for Loc {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE_ARG: Regex = Regex::new("^𝛼?(\\d+)$").unwrap();
            static ref RE_OBJ: Regex = Regex::new("^ν(\\d+)$").unwrap();
        }
        if let Some(caps) = RE_ARG.captures(s) {
            Ok(Loc::Attr(
                caps.get(1).unwrap().as_str().parse::<i8>().unwrap(),
            ))
        } else if let Some(caps) = RE_OBJ.captures(s) {
            Ok(Loc::Obj(
                caps.get(1).unwrap().as_str().parse::<Ob>().unwrap(),
            ))
        } else {
            match s {
                "Φ" | "Q" => Ok(Loc::Root),
                "Δ" | "D" => Ok(Loc::Delta),
                "𝜋" | "P" => Ok(Loc::Pi),
                "ρ" | "^" => Ok(Loc::Rho),
                "𝜑" | "@" => Ok(Loc::Phi),
                "σ" | "&" => Ok(Loc::Sigma),
                _ => Err(format!("Unknown loc: '{}'", s)),
            }
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self {
            Loc::Root => "Φ".to_owned(),
            Loc::Rho => "ρ".to_owned(),
            Loc::Delta => "Δ".to_owned(),
            Loc::Phi => "𝜑".to_owned(),
            Loc::Pi => "𝜋".to_owned(),
            Loc::Sigma => "σ".to_owned(),
            Loc::Attr(i) => format!("𝛼{}", i),
            Loc::Obj(i) => format!("ν{}", i),
        })
    }
}

#[rstest]
#[case("Q")]
#[case("&")]
#[case("^")]
#[case("@")]
#[case("D")]
#[case("Δ")]
#[case("ν78")]
#[case("𝜑")]
#[case("𝜋")]
#[case("𝛼0")]
#[case("σ")]
#[case("ρ")]
pub fn parses_and_prints(#[case] txt: &str) {
    let loc1 = Loc::from_str(txt).unwrap();
    let loc2 = Loc::from_str(&loc1.to_string()).unwrap();
    assert_eq!(loc1, loc2)
}
