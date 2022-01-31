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

use crate::path::{Path, Item};
use crate::Data;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Default, Clone)]
pub struct Object {
    pub data: Data,
    pub atom: Atom,
    pub phi: Option<usize>,
    pub rho: Option<usize>,
    pub kids: HashMap<Item, Path>
}

impl Object {
    pub fn empty() -> Obs {
        Obs { ..Default::default() }
    }

    pub fn push_atom(mut self, a: usize) -> Obs {
        self.atom = Some(a);
        self
    }

    pub fn push_data(mut self, d: Data) -> Obs {
        self.data = Some(d);
        self
    }

    pub fn push(mut self, i: Item, p: Path) -> Obs {
        self.kids.insert(i, p);
        self
    }
}

#[test]
fn makes_simple_obs() {
    let obs = Obs::empty()
        .push_atom(1)
        .push_data(44)
        .push(Item::from_str("^").unwrap(), Path::from_str("v4").unwrap());
    assert_eq!(obs.atom.unwrap(), 1);
    assert_eq!(obs.data.unwrap(), 42);
    assert_eq!(obs.kids.len(), 1)
}
