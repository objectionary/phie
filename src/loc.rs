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
pub enum Loc {
    Root,
    Rho,
    Phi,
    Xi,
    Psi,
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
            static ref RE_OBJ: Regex = Regex::new("^[v|ν](\\d+)$").unwrap();
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
                "Φ" | "R" => Ok(Loc::Root),
                "𝜓" | "P" => Ok(Loc::Psi),
                "Δ" | "D" => Ok(Loc::Delta),
                "ρ" | "^" => Ok(Loc::Rho),
                "ξ" | "$" => Ok(Loc::Xi),
                "φ" | "@" => Ok(Loc::Phi),
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
            Loc::Psi => "𝜓".to_owned(),
            Loc::Phi => "φ".to_owned(),
            Loc::Xi => "ξ".to_owned(),
            Loc::Sigma => "σ".to_owned(),
            Loc::Attr(i) => format!("𝛼{}", i),
            Loc::Obj(i) => format!("ν{}", i),
        })
    }
}

#[rstest]
#[case("R")]
#[case("&")]
#[case("$")]
#[case("^")]
#[case("@")]
#[case("D")]
#[case("P")]
#[case("𝜓")]
#[case("Δ")]
#[case("v78")]
#[case("Φ")]
#[case("𝛼0")]
#[case("σ")]
#[case("ρ")]
pub fn parses_and_prints(#[case] txt: &str) {
    let loc1 = Loc::from_str(&txt).unwrap();
    let loc2 = Loc::from_str(&loc1.to_string()).unwrap();
    assert_eq!(loc1, loc2)
}
