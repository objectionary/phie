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
use crate::atoms::Directive;
use crate::objects::Obs;
use crate::path::{Item, Path};
use crate::ph;
use crate::Data;
use lazy_static::lazy_static;
use std::str::FromStr;

#[derive(Default, Clone)]
pub struct Emu {
    pub obses: Vec<Obs>,
    pub registers: [i64; 16],
    pub stack: Vec<usize>,
    pub atoms: Vec<Atom>,
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional OBSes.
    pub fn empty() -> Emu {
        Emu {
            ..Default::default()
        }
    }

    /// Add an additional OBS
    pub fn push(&mut self, obs: Obs) -> &mut Emu {
        self.obses.push(obs);
        self
    }

    /// Add an additional atom
    pub fn push_atom(&mut self, atom: Atom) -> &mut Emu {
        self.atoms.push(atom);
        self
    }

    /// Add an additional OBS to the stack.
    pub fn push_stack(&mut self, id: usize) -> &mut Emu {
        self.stack.push(id);
        self
    }

    /// Perform "dataization" procedure on a single OBS and return
    /// the data found there. The position of the OBS to be dataized
    /// is taken from the stack.
    pub fn dataize(&mut self) {
        let pos = self.stack.last().unwrap();
        let obs = &self.obses[pos];
        if obs.atom.is_some() {
            let data = self.exec(obs.atom.unwrap());
            obs.ret = Some(data)
        } else if obs.data.is_some() {
            obs.ret = obs.data
        } else if obs.kids.contains_key(&Item::Phi) {
            let phi = self.find(obs.kids.get(&Item::Phi).unwrap()).unwrap();
            std::mem::replace(&mut self.stack[pos], phi);
            self.dataize()
        } else {
            panic!("Can't dataize an empty OBS #{}", pos)
        }
    }

    /// Execute a single atom by the `id` and return the data it
    /// produces.
    fn exec(&mut self, id: usize) -> Data {
        let atom = &self.atoms[id];
        let mut ip = 0;
        loop {
            let dir = atom.dir(ip).unwrap();
            match dir {
                Directive::LABEL(_) => {}
                Directive::DATAIZE(path) => {
                    let obs = self.find(path).unwrap();
                    self.stack.push(obs);
                    self.dataize();
                    self.stack.pop()
                }
                Directive::RETURN(reg) => return self.registers[reg.num()],
                Directive::JUMP(label, reg, cond) => {
                    if cond.is_true(self.registers[reg.num()]) {
                        ip = atom.label_position(label).unwrap().0;
                    }
                }
                Directive::SUB(left, right, reg) => {
                    self.registers[reg.num()] = self.obses[self.find(right).unwrap()].ret -
                        self.obses[self.find(left).unwrap()].ret;
                }
                Directive::ADD(left, right, reg) => {
                    self.registers[reg.num()] = self.obses[self.find(right).unwrap()].ret +
                        self.obses[self.find(left).unwrap()].ret;
                }
                Directive::SAVE(reg, path) => {
                    self.obses[self.find(left).unwrap()].ret
                }
                Directive::LOAD(path, reg) => panic!("LOAD not implemented yet"),
            }
        }
    }

    /// Suppose, the incoming path is `^.0.@.2`. We have to find the right
    /// OBS in the catalog of them and return the position of the found one.
    fn find(&self, path: &Path) -> Option<usize> {
        let mut items = path.to_vec();
        let item = items.remove(0);
        let mut obs = self.obses.get(self.stack.last());
        let next = match item {
            Item::Xi => obs,
            Item::Rho => obs.kids.get(Item::Rho),
            _ => panic!("What?")
        };
        if items.is_empty() {
            return next;
        }
        return self.obs_by_path()
    }
}

#[test]
#[should_panic]
pub fn panics_on_empty_obs() {
    let mut emu = Emu::empty();
    emu.push(Obs::empty());
    emu.push_stack(0);
    emu.dataize();
}

#[test]
pub fn dataize_simple_data() {
    let mut emu = Emu::empty();
    emu.push(Obs::empty().push_data(42));
    emu.push_stack(0);
    assert_eq!(42, emu.dataize());
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.push(Obs::empty().push_data(42))
        .push(Obs::empty().push(Item::Phi, "v0".parse().unwrap()));
    emu.push_stack(1);
    assert_eq!(42, emu.dataize());
}

#[test]
pub fn exec_simple_atom() {
    let mut emu = Emu::empty();
    emu.push(Obs::empty().push_data(42))
        .push(Obs::empty().push_data(7))
        .push(
            Obs::empty()
                .push_atom(0)
                .push(Item::Rho, ph!("v0"))
                .push(Item::Arg(0), ph!("v1")))
        .push_atom(
            "
            ADD ^ AND $.1 TO #0
            RETURN #0
            ".parse().unwrap(),
        );
    emu.push_stack(2);
    assert_eq!(0, emu.dataize());
}
