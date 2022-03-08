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
        Basket {
            ob: 0,
            psi: -1,
            kids: HashMap::new(),
        }
    }

    pub fn start(ob: Ob, psi: Bk) -> Basket {
        Basket {
            ob,
            psi,
            kids: HashMap::new(),
        }
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
            self.kids
                .iter()
                .map(|(i, d)| format!("{}{}", i, d))
                .sorted()
                .collect::<Vec<String>>(),
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
        let re = Regex::new("\\[(.*)]").unwrap();
        let mut bsk = Basket::empty();
        let parts: Vec<&str> = re
            .captures(s)
            .expect(format!("Can't parse the basket: '{}'", s).as_str())
            .get(1)
            .expect(format!("Can't find the matcher inside '{}'", s).as_str())
            .as_str()
            .trim()
            .split(",")
            .map(|t| t.trim())
            .collect();
        let ob: String = parts.get(0).unwrap().chars().skip(1).collect();
        bsk.ob = ob.parse().expect("Can't parse the v part");
        let psi: String = parts.get(1).unwrap().chars().skip(3).collect();
        bsk.psi = psi.parse().expect("Can't parse the psi part");
        let pre = Regex::new("^(.*)(⇶0x|⇉β|→\\(ν|→∅|→\\?)(.*?)\\)?$").unwrap();
        for p in parts.iter().skip(2) {
            let caps = pre.captures(p).unwrap();
            let kid = match caps.get(2).unwrap().as_str() {
                "→∅" => Kid::Empt,
                "⇶0x" => {
                    let data = caps.get(3).unwrap().as_str();
                    Kid::Dtzd(
                        Data::from_str_radix(data, 16)
                            .expect(format!("Can't parse data '{}'", data).as_str()),
                    )
                }
                "⇉β" => {
                    let (b, a) = caps
                        .get(3)
                        .unwrap()
                        .as_str()
                        .split(".")
                        .collect_tuple()
                        .unwrap();
                    Kid::Wait(b.parse().unwrap(), Loc::from_str(a).unwrap())
                }
                "→(ν" => {
                    let part = caps.get(3).unwrap().as_str();
                    let (o, p) = part
                        .split(";")
                        .collect_tuple()
                        .expect(format!("Can't parse the needed pair '{}'", part).as_str());
                    let psi: String = p.chars().skip(1).collect();
                    Kid::Need(o.parse().unwrap(), psi.parse().unwrap())
                }
                "→?" => Kid::Rqtd,
                _ => panic!("Oops"),
            };
            bsk.kids
                .insert(Loc::from_str(caps.get(1).unwrap().as_str()).unwrap(), kid);
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
fn prints_itself() {
    let mut basket = Basket::start(5, 7);
    basket.put(Loc::Delta, Kid::Dtzd(42));
    basket.put(Loc::Rho, Kid::Wait(42, Loc::Phi));
    basket.put(Loc::Attr(1), Kid::Need(7, 12));
    assert_eq!(
        "[ν5, ξ:β7, Δ⇶0x002A, ρ⇉β42.φ, 𝛼1→(ν7;β12)]",
        basket.to_string()
    );
}

#[test]
fn parses_itself() {
    let txt = "[ν5, ξ:β18, Δ⇶0x1F21, ρ⇉β4.φ, φ→∅, 𝛼12→?, 𝛼1→?, 𝛼3→(ν5;β5)]";
    let basket = Basket::from_str(txt).unwrap();
    assert_eq!(txt, basket.to_string());
}

#[rstest]
#[case("[ν5, ξ:β7, Δ⇶0x002A, ρ⇉β42.φ]")]
#[case("[ν5, ξ:β18, Δ⇶0x1F21, ρ⇉β4.φ, φ→∅, 𝛼12→?, 𝛼1→?, 𝛼3→(ν5;β5)]")]
fn parses_text(#[case] txt: &str) {
    let basket = Basket::from_str(txt).unwrap();
    assert_eq!(txt, basket.to_string());
}
