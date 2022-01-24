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

use crate::atoms::Atom;
use crate::Data;
use crate::obs::Obs;
use crate::atoms::Directive;
use crate::path::{Item, Path};
use crate::ph;
use lazy_static::lazy_static;
use std::str::FromStr;

#[derive(Default)]
pub struct Emu {
    pub obses: Vec<Obs>,
    pub registers: [i64; 16],
    pub stack: Vec<i16>,
    pub atoms: Vec<Atom>
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional OBSes.
    pub fn empty() -> Emu {
        Emu {
            ..Default::default()
        }
    }

    /// Make a copy of the Emu, with an additional OBS
    pub fn with(self, obs: Obs) -> Emu {
        Emu {
            obses: [self.obses, vec![obs]].concat(),
            ..self
        }
    }

    /// Make a copy of the Emu, with an additional atom
    pub fn with_atom(self, atom: Atom) -> Emu {
        Emu {
            atoms: [self.atoms, vec![atom]].concat(),
            ..self
        }
    }

    /// Perform "dataization" procedure on a single OBS and return
    /// the data found there.
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

    fn exec(&mut self, id: usize) -> Data {
        let atom = &self.atoms[id];
        let mut ip = 0;
        loop {
            let dir = atom.dir(ip).unwrap();
            match dir {
                Directive::LABEL(_) => {},
                Directive::DATAIZE(path) => panic!("DATAIZE not implemented yet"),
                Directive::RETURN(reg) => return self.registers[reg.num()],
                Directive::JUMP(label, reg, cond) => {
                    if cond.is_true(self.registers[reg.num()]) {
                        ip = atom.label_position(label).unwrap().0;
                    }
                },
                Directive::WRITE(data, reg) => {
                    self.registers[reg.num()] = *data;
                },
                Directive::SUB(left, right, reg) => {
                    self.registers[reg.num()] = 0;
                },
                Directive::ADD(left, right, reg) => {
                    self.registers[reg.num()] = 0;
                },
                Directive::SAVE(reg, path) => panic!("SAVE not implemented yet"),
                Directive::LOAD(path, reg) => panic!("LOAD not implemented yet")
            }
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

#[test]
pub fn exec_simple_atom() {
    let emu = Emu::empty()
        .with(Obs::Data(ph!("R"), 42))
        .with(Obs::Data(ph!("R"), 7))
        .with(Obs::Atom(0, ph!("v0"), vec![ph!("v1")]))
        .with_atom(
            Atom::from_str(
                "
                ADD ^.0 AND ^.1 TO #0
                RETURN #0
                ",
            )
            .unwrap(),
        );
    assert_eq!(0, emu.dataize(2));
}
