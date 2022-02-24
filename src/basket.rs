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
use std::collections::HashMap;
use std::fmt;

pub type Bk = isize;

pub enum Kid {
    Empty,
    Requested,
    Waiting(Bk),
    Dataized(Data),
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

    pub fn request(&mut self, loc: Loc) {
        self.kids.insert(loc, Kid::Requested);
    }

    pub fn wait(&mut self, loc: Loc, bk: Bk) {
        self.kids.insert(loc, Kid::Waiting(bk));
    }

    pub fn dataize(&mut self, loc: Loc, d: Data) {
        self.kids.insert(loc, Kid::Dataized(d));
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
            Kid::Empty => "→?".to_string(),
            Kid::Requested => "→!".to_string(),
            Kid::Waiting(bk) => format!("⇉β{}.φ", bk),
            Kid::Dataized(d) => format!("⇶0x{:04X}", d),
        })
    }
}

#[test]
fn makes_simple_basket() {
    let mut basket = Basket::start(0, 0);
    basket.dataize(Loc::Delta, 42);
    if let Kid::Dataized(d) = basket.kids.get(&Loc::Delta).unwrap() {
        assert_eq!(42, *d);
    }
}

#[test]
fn prints_itself() {
    let mut basket = Basket::start(5, 7);
    basket.dataize(Loc::Delta, 42);
    basket.wait(Loc::Rho, 42);
    assert_eq!("[ν5, ξ:β7, Δ⇶0x002A, ρ⇉β42.φ]", basket.to_string());
}
