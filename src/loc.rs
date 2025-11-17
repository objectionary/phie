// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use std::{fmt, str::FromStr};

use regex::Regex;
use rstest::rstest;

use crate::object::Ob;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Loc {
    Root,
    Rho,
    Phi,
    Pi,
    Delta,
    Sigma,
    Attr(i8),
    Obj(Ob)
}

impl FromStr for Loc {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re_arg = Regex::new("^𝛼?(\\d+)$")
            .map_err(|e| format!("Invalid RE_ARG regex pattern: {}", e))?;
        let re_obj =
            Regex::new("^ν(\\d+)$").map_err(|e| format!("Invalid RE_OBJ regex pattern: {}", e))?;

        if let Some(caps) = re_arg.captures(s) {
            let attr_str = caps
                .get(1)
                .ok_or_else(|| format!("Missing capture group in attr pattern: '{}'", s))?
                .as_str();
            let attr_num = attr_str
                .parse::<i8>()
                .map_err(|e| format!("Failed to parse attr number '{}': {}", attr_str, e))?;
            Ok(Loc::Attr(attr_num))
        } else if let Some(caps) = re_obj.captures(s) {
            let obj_str = caps
                .get(1)
                .ok_or_else(|| format!("Missing capture group in obj pattern: '{}'", s))?
                .as_str();
            let obj_num = obj_str
                .parse::<Ob>()
                .map_err(|e| format!("Failed to parse obj number '{}': {}", obj_str, e))?;
            Ok(Loc::Obj(obj_num))
        } else {
            match s {
                "Φ" | "Q" => Ok(Loc::Root),
                "Δ" | "D" => Ok(Loc::Delta),
                "𝜋" | "P" => Ok(Loc::Pi),
                "ρ" | "^" => Ok(Loc::Rho),
                "𝜑" | "@" => Ok(Loc::Phi),
                "σ" | "&" => Ok(Loc::Sigma),
                _ => Err(format!("Unknown loc: '{}'", s))
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
            Loc::Obj(i) => format!("ν{}", i)
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

#[test]
fn fails_on_unknown_loc() {
    let result = Loc::from_str("unknown");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Unknown loc"));
}

#[test]
fn fails_on_invalid_attr_number() {
    let result = Loc::from_str("𝛼999999999999");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Failed to parse attr number"));
}

#[test]
fn fails_on_invalid_obj_number() {
    let result = Loc::from_str("ν99999999999999999999999999");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Failed to parse obj number"));
}
