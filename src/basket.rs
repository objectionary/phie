// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use crate::data::Data;
use crate::loc::Loc;
use crate::object::Ob;
use itertools::Itertools;
use regex::Regex;
use rstest::rstest;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

pub type Bk = isize;

pub enum Kid {
    Empt,
    Rqtd,
    Need(Ob, Bk),
    Wait(Bk, Loc),
    Dtzd(Data),
}

pub struct Basket {
    pub ob: Ob,
    pub psi: Bk,
    pub kids: HashMap<Loc, Kid>,
}

impl Basket {
    pub fn empty() -> Basket {
        Basket { ob: 0, psi: -1, kids: HashMap::new() }
    }

    pub fn start(ob: Ob, psi: Bk) -> Basket {
        Basket { ob, psi, kids: HashMap::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.psi < 0
    }

    pub fn put(&mut self, loc: Loc, kid: Kid) {
        self.kids.insert(loc, kid);
    }
}

impl fmt::Display for Basket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = vec![];
        parts.push(format!("ν{}", self.ob));
        parts.push(format!("ξ:β{}", self.psi));
        parts.extend(
            self.kids.iter().map(|(i, d)| format!("{}{}", i, d)).sorted().collect::<Vec<String>>(),
        );
        write!(f, "[{}]", parts.iter().join(", "))
    }
}

impl fmt::Display for Kid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&match self {
            Kid::Empt => "→∅".to_string(),
            Kid::Rqtd => "→?".to_string(),
            Kid::Need(ob, bk) => format!("→(ν{};β{})", ob, bk),
            Kid::Wait(bk, loc) => format!("⇉β{}.{}", bk, loc),
            Kid::Dtzd(d) => format!("⇶0x{:04X}", d),
        })
    }
}

impl FromStr for Basket {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re =
            Regex::new("\\[(.*)]").map_err(|e| format!("Invalid basket regex pattern: {}", e))?;
        let mut bsk = Basket::empty();
        let caps = re.captures(s).ok_or_else(|| format!("Can't parse the basket: '{}'", s))?;
        let inner =
            caps.get(1).ok_or_else(|| format!("Can't find the matcher inside '{}'", s))?.as_str();
        let parts: Vec<&str> = inner.trim().split(',').map(|t| t.trim()).collect();
        let ob_str: String = parts
            .first()
            .ok_or_else(|| format!("Empty basket content in '{}'", s))?
            .chars()
            .skip(1)
            .collect();
        bsk.ob =
            ob_str.parse().map_err(|e| format!("Can't parse the v part '{}': {}", ob_str, e))?;
        let psi_str: String = parts
            .get(1)
            .ok_or_else(|| format!("Missing psi part in basket '{}'", s))?
            .chars()
            .skip(3)
            .collect();
        bsk.psi = psi_str
            .parse()
            .map_err(|e| format!("Can't parse the psi part '{}': {}", psi_str, e))?;
        let pre = Regex::new("^(.*)(⇶0x|⇉β|→\\(ν|→∅|→\\?)(.*?)\\)?$")
            .map_err(|e| format!("Invalid kid pattern regex: {}", e))?;
        for p in parts.iter().skip(2) {
            let caps =
                pre.captures(p).ok_or_else(|| format!("Can't parse kid pattern in '{}'", p))?;
            let kind_str =
                caps.get(2).ok_or_else(|| format!("Missing kid type in '{}'", p))?.as_str();
            let kid = match kind_str {
                "→∅" => Kid::Empt,
                "⇶0x" => {
                    let data = caps
                        .get(3)
                        .ok_or_else(|| format!("Missing data value in '{}'", p))?
                        .as_str();
                    let parsed_data = Data::from_str_radix(data, 16)
                        .map_err(|e| format!("Can't parse data '{}': {}", data, e))?;
                    Kid::Dtzd(parsed_data)
                }
                "⇉β" => {
                    let wait_str = caps
                        .get(3)
                        .ok_or_else(|| format!("Missing wait value in '{}'", p))?
                        .as_str();
                    let (b, a) = wait_str
                        .split('.')
                        .collect_tuple()
                        .ok_or_else(|| format!("Invalid wait format in '{}'", wait_str))?;
                    let b_num = b
                        .parse()
                        .map_err(|e| format!("Can't parse wait number '{}': {}", b, e))?;
                    let a_loc = Loc::from_str(a)
                        .map_err(|e| format!("Can't parse wait loc '{}': {}", a, e))?;
                    Kid::Wait(b_num, a_loc)
                }
                "→(ν" => {
                    let part = caps
                        .get(3)
                        .ok_or_else(|| format!("Missing need value in '{}'", p))?
                        .as_str();
                    let (o, p) = part
                        .split(';')
                        .collect_tuple()
                        .ok_or_else(|| format!("Can't parse the needed pair '{}'", part))?;
                    let psi_str: String = p.chars().skip(1).collect();
                    let o_num = o
                        .parse()
                        .map_err(|e| format!("Can't parse need obj '{}': {}", o, e))?;
                    let psi_num = psi_str
                        .parse()
                        .map_err(|e| format!("Can't parse need psi '{}': {}", psi_str, e))?;
                    Kid::Need(o_num, psi_num)
                }
                "→?" => Kid::Rqtd,
                _ => return Err(format!("Unknown kid type: '{}'", kind_str)),
            };
            let loc_str = caps
                .get(1)
                .ok_or_else(|| format!("Missing location in '{}'", p))?
                .as_str();
            let loc = Loc::from_str(loc_str)
                .map_err(|e| format!("Can't parse location '{}': {}", loc_str, e))?;
            bsk.kids.insert(loc, kid);
        }
        Ok(bsk)
    }
}

#[test]
fn makes_simple_basket() {
    let mut basket = Basket::start(0, 0);
    basket.put(Loc::Delta, Kid::Dtzd(42));
    if let Kid::Dtzd(d) = basket.kids.get(&Loc::Delta).unwrap() {
        assert_eq!(42, *d);
    }
}

#[test]
fn checks_if_empty() {
    let empty = Basket::empty();
    assert!(empty.is_empty());
    let not_empty = Basket::start(0, 0);
    assert!(!not_empty.is_empty());
}

#[test]
fn prints_itself() {
    let mut basket = Basket::start(5, 7);
    basket.put(Loc::Delta, Kid::Dtzd(42));
    basket.put(Loc::Rho, Kid::Wait(42, Loc::Phi));
    basket.put(Loc::Attr(1), Kid::Need(7, 12));
    assert_eq!(
        "[ν5, ξ:β7, Δ⇶0x002A, ρ⇉β42.𝜑, 𝛼1→(ν7;β12)]",
        basket.to_string()
    );
}

#[test]
fn parses_itself() {
    let txt = "[ν5, ξ:β18, Δ⇶0x1F21, ρ⇉β4.𝜑, 𝛼12→?, 𝛼1→?, 𝛼3→(ν5;β5), 𝜑→∅]";
    let basket = Basket::from_str(txt).unwrap();
    assert_eq!(txt, basket.to_string());
}

#[test]
fn fails_on_invalid_basket_format() {
    let result = Basket::from_str("invalid");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse the basket"));
}

#[test]
fn fails_on_invalid_data_hex() {
    let result = Basket::from_str("[ν5, ξ:β7, Δ⇶0xZZZZ]");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse data"));
}

#[test]
fn fails_on_invalid_wait_format() {
    let result = Basket::from_str("[ν5, ξ:β7, ρ⇉βinvalid]");
    assert!(result.is_err());
}

#[test]
fn fails_on_invalid_need_format() {
    let result = Basket::from_str("[ν5, ξ:β7, 𝛼1→(νinvalid)]");
    assert!(result.is_err());
}

#[test]
fn fails_on_unknown_kid_type() {
    let result = Basket::from_str("[ν5, ξ:β7, 𝛼1→☠]");
    assert!(result.is_err());
}

#[test]
fn fails_on_invalid_need_obj_number() {
    let result = Basket::from_str("[ν5, ξ:β7, 𝛼1→(νinvalid_obj;β5)]");
    assert!(result.is_err());
}

#[test]
fn fails_on_invalid_need_psi_number() {
    let result = Basket::from_str("[ν5, ξ:β7, 𝛼1→(ν5;βinvalid_psi)]");
    assert!(result.is_err());
}

#[test]
fn fails_on_invalid_location_in_kid() {
    let result = Basket::from_str("[ν5, ξ:β7, invalid_loc→?]");
    assert!(result.is_err());
}

#[test]
fn fails_on_invalid_ob_number() {
    let result = Basket::from_str("[νinvalid, ξ:β7]");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse the v part"));
}

#[test]
fn fails_on_invalid_psi_number() {
    let result = Basket::from_str("[ν5, ξ:βinvalid]");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse the psi part"));
}

#[test]
fn fails_on_missing_psi_part() {
    let result = Basket::from_str("[ν5]");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Missing psi part"));
}

#[test]
fn fails_on_invalid_wait_number() {
    let result = Basket::from_str("[ν5, ξ:β7, ρ⇉βnotnum.𝜑]");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse wait number"));
}

#[test]
fn fails_on_invalid_wait_loc() {
    let result = Basket::from_str("[ν5, ξ:β7, ρ⇉β5.invalid]");
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.contains("Can't parse wait loc"));
}

#[rstest]
#[case("[ν5, ξ:β7, Δ⇶0x002A, ρ⇉β42.𝜑]")]
#[case("[ν5, ξ:β18, Δ⇶0x1F21, ρ⇉β4.𝜑, 𝛼12→?, 𝛼1→?, 𝛼3→(ν5;β5), 𝜑→∅]")]
fn parses_text(#[case] txt: &str) {
    let basket = Basket::from_str(txt).unwrap();
    assert_eq!(txt, basket.to_string());
}
