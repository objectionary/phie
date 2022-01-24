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

use crate::directives::Directives;
use crate::obs::Obs;
use crate::path::{Item, Path};
use crate::primitives::Data;

#[derive(Default)]
pub struct Emu {
    pub obses: Vec<Obs>,
    pub registers: [i64; 16],
    pub stack: Vec<i16>,
    pub directives: Directives,
}

impl Emu {
    pub fn empty() -> Emu {
        Emu {
            ..Default::default()
        }
    }

    pub fn with(self, obs: Obs) -> Emu {
        Emu {
            obses: [self.obses, vec![obs]].concat(),
            ..self
        }
    }

    pub fn dataize(&self, id: usize) -> Data {
        let obs = &self.obses[id];
        match obs {
            Obs::Empty => {
                panic!("Can't dataize an empty object")
            }
            Obs::Data(sup, data) => *data,
            Obs::Abstract(phi, args) => match phi.item(0).unwrap() {
                Item::Obs(id) => self.dataize(*id),
                _ => panic!("Invalid path"),
            },
            Obs::Atom(id, rho, args) => 0,
            Obs::Copy(rho, args) => 0,
        }
    }
}

#[test]
pub fn dataize_simple_data() {
    let emu = Emu::empty().with(Obs::Data("R".parse().unwrap(), 42));
    assert_eq!(42, emu.dataize(0));
}

#[test]
pub fn with_simple_decorator() {
    let emu = Emu::empty()
        .with(Obs::Data("R".parse().unwrap(), 42))
        .with(Obs::Abstract("v0".parse().unwrap(), vec![]));
    assert_eq!(42, emu.dataize(1));
}
