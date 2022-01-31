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

use crate::path::Path;
use crate::register::Register;
use crate::Data;
use itertools::Itertools;
use std::str::{self, FromStr};
use strum_macros::EnumString;

#[derive(Debug, Clone)]
pub struct DirectiveError {
    msg: String,
}

#[derive(Clone, EnumString)]
pub enum Condition {
    LTZ,
    GTZ,
    GTEZ,
    LTEZ,
    EQZ,
}

#[derive(Clone)]
pub enum Directive {
    /// Anchor point for future JUMPs
    LABEL(String),

    /// Dataize the OBS by this path
    DATAIZE(Path),
    RETURN(Register),
    JUMP(String, Register, Condition),
    LOAD(Path, Register),
    SAVE(Register, Path),
    SUB(Path, Path, Register),
    ADD(Path, Path, Register),
}

#[derive(Default, Clone)]
pub struct Atom {
    dirs: Vec<Directive>,
}

impl str::FromStr for Directive {
    type Err = DirectiveError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split(" ").filter(|&t| !t.is_empty()).collect();
        let start: &str = parts.get(0).unwrap();
        if start.ends_with(":") {
            return Ok(Directive::LABEL(
                String::from_str(&start[0..start.len() - 1]).unwrap(),
            ));
        }
        match parts[0] {
            "DATAIZE" => Ok(Directive::DATAIZE(
                Path::from_str(parts.get(1).unwrap()).unwrap(),
            )),
            "RETURN" => Ok(Directive::RETURN(
                Register::from_str(parts.get(1).unwrap()).unwrap(),
            )),
            "JUMP" => Ok(Directive::JUMP(
                String::from_str(parts.get(1).unwrap()).unwrap(),
                Register::from_str(parts.get(3).unwrap()).unwrap(),
                Condition::from_str(parts.get(4).unwrap()).unwrap(),
            )),
            "LOAD" => Ok(Directive::LOAD(
                Path::from_str(parts.get(1).unwrap()).unwrap(),
                Register::from_str(parts.get(3).unwrap()).unwrap(),
            )),
            "SAVE" => Ok(Directive::SAVE(
                Register::from_str(parts.get(3).unwrap()).unwrap(),
                Path::from_str(parts.get(1).unwrap()).unwrap(),
            )),
            "ADD" => Ok(Directive::ADD(
                Path::from_str(parts.get(1).unwrap()).unwrap(),
                Path::from_str(parts.get(3).unwrap()).unwrap(),
                Register::from_str(parts.get(5).unwrap()).unwrap(),
            )),
            "SUB" => Ok(Directive::SUB(
                Path::from_str(parts.get(1).unwrap()).unwrap(),
                Path::from_str(parts.get(3).unwrap()).unwrap(),
                Register::from_str(parts.get(5).unwrap()).unwrap(),
            )),
            _ => Err(DirectiveError {
                msg: format!("Invalid directive start: '{}'", start),
            }),
        }
    }
}

impl str::FromStr for Atom {
    type Err = DirectiveError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Atom {
            dirs: s
                .trim()
                .split("\n")
                .filter(|&t| !t.is_empty())
                .collect::<Vec<&str>>()
                .iter()
                .map(|t| Directive::from_str(t).unwrap())
                .collect(),
        })
    }
}

impl Atom {
    /// Get one directive by the index `idx`.
    pub fn dir(&self, idx: usize) -> Option<&Directive> {
        self.dirs.get(idx)
    }

    /// Find the position of the provided label.
    pub fn label_position(&self, txt: &str) -> Option<(usize, &Directive)> {
        self.dirs
            .iter()
            .find_position(|d| matches!(d, Directive::LABEL(t) if t == txt))
    }
}

impl Condition {
    pub fn is_true(&self, val: i64) -> bool {
        match self {
            Condition::GTZ => val > 0,
            Condition::GTEZ => val >= 0,
            Condition::LTZ => val < 0,
            Condition::LTEZ => val <= 0,
            Condition::EQZ => val == 0,
        }
    }
}

#[test]
fn parses_return_directive() {
    let dir = Directive::from_str("RETURN   #A ").unwrap();
    match dir {
        Directive::RETURN(r) => {
            assert_eq!(r, Register::from_str("#A").unwrap());
            assert_eq!(r.num(), 10)
        }
        _ => panic!("Wrong type"),
    }
}

#[test]
fn finds_label_position() {
    let atom = Atom::from_str("RETURN #9\ntest:\nRETURN #1").unwrap();
    assert_eq!(atom.label_position("test").unwrap().0, 1);
}

#[test]
fn parses_simple_atom() {
    let atom = Atom::from_str(
        "

        DATAIZE $.2
        ADD ^.0 AND $.1.0.^.3 TO #3
        JUMP  test   IF #3 EQZ
        RETURN #0

        test:
        WRITE 65536 TO #C
        READ ^.^.&.@.6 TO #B
        RETURN #1
        ",
    )
    .unwrap();
    assert!(matches!(atom.dir(0).unwrap(), Directive::DATAIZE(_)));
    assert!(matches!(atom.dir(1).unwrap(), Directive::ADD(_, _, _)));
    assert!(matches!(atom.dir(2).unwrap(), Directive::JUMP(_, _, _)));
    assert!(matches!(atom.dir(3).unwrap(), Directive::RETURN(_)))
}
