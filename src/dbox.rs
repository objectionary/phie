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
use crate::path::Item;
use crate::object::Ob;
use std::collections::HashMap;
use std::fmt;
use itertools::Itertools;

pub type Bx = usize;

pub struct Dbox {
    pub ob: Ob,
    pub xi: Bx,
    pub psi: i8,
    ret: Option<Data>,
    kids: HashMap<Item, Data>,
}

impl Dbox {
    pub fn empty() -> Dbox {
        Dbox {
            ob: 0,
            xi: 0,
            psi: 0,
            ret: None,
            kids: HashMap::new(),
        }
    }

    pub fn start(ob: Ob, xi: Bx, psi: i8) -> Dbox {
        Dbox {
            ob,
            xi,
            psi,
            ret: None,
            kids: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ob <= 0
    }

    pub fn put_xi(&mut self, xi: Bx) {
        self.xi = xi
    }

    pub fn put_ret(&mut self, ret: Data) {
        self.ret = Some(ret)
    }

    pub fn put_kid(&mut self, item: Item, d: Data) {
        self.kids.insert(item, d);
    }
}

impl fmt::Display for Dbox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = vec![];
        parts.push(format!("ν{}", self.ob));
        parts.push(format!("ξ:#{}", self.xi));
        if let Some(r) = self.ret {
            parts.push(format!("r:0x{:04X}", r));
        }
        if self.psi > 0 {
            parts.push(format!("𝜓{}", self.psi));
        }
        parts.extend(
            self.kids.iter()
                .map(|(i, d)| format!("{}:0x{:04X}", i, d))
                .sorted()
                .collect::<Vec<String>>()
        );
        write!(f, "[{}]", parts.iter().join(", "))
    }
}

#[test]
fn makes_simple_dabox() {
    let mut dabox = Dbox::start(0, 0, 0);
    dabox.put_ret(42);
    assert_eq!(42, dabox.ret.unwrap());
}

#[test]
fn prints_itself() {
    let mut dabox = Dbox::start(5, 7, 0);
    dabox.put_ret(42);
    dabox.put_kid(Item::Rho, 42);
    assert_eq!("[ν5, ξ:#7, r:0x002A, ρ:0x002A]", dabox.to_string());
}
