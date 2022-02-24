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

use crate::basket::{Basket, Bk, Kid};
use crate::data::Data;
use crate::loc::Loc;
use crate::locator::Locator;
use crate::object::{Ob, Object};
use arr_macro::arr;
use itertools::Itertools;
use log::trace;
use regex::Regex;
use std::fmt;
use std::str::FromStr;

pub const ROOT_BK: Bk = 0;
pub const ROOT_OB: Ob = 0;

pub struct Emu {
    pub objects: [Object; 32],
    pub baskets: [Basket; 128],
}

macro_rules! join {
    ($log:expr) => {
        $log.iter().join("; ")
    };
}

impl fmt::Display for Emu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];
        for i in self.objects.iter().enumerate() {
            let (ob, obj): (usize, &Object) = i;
            if obj.is_empty() {
                continue;
            }
            lines.push(format!(
                "ν{} {}{}",
                ob,
                obj,
                self.baskets
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| !d.is_empty() && d.ob as usize == ob)
                    .map(|(i, d)| format!("\n\t➞ β{} {}", i, d))
                    .collect::<Vec<String>>()
                    .join("")
            ));
        }
        f.write_str(lines.join("\n").as_str())
    }
}

impl FromStr for Emu {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut emu = Emu::empty();
        let re_line = Regex::new("ν(\\d+) ↦ (⟦.*⟧)").unwrap();
        for line in s.trim().split("\n").map(|t| t.trim()) {
            let caps = re_line.captures(line).unwrap();
            let v: Ob = caps.get(1).unwrap().as_str().parse().unwrap();
            emu.put(v, Object::from_str(caps.get(2).unwrap().as_str()).unwrap());
        }
        Ok(emu)
    }
}

#[macro_export]
macro_rules! assert_emu {
    ($eq:expr, $txt:expr) => {
        let mut emu: Emu = $txt.parse().unwrap();
        assert_eq!(
            $eq,
            emu.cycle().0,
            "The expected dataization result is {}",
            $eq
        );
    };
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional objects.
    pub fn empty() -> Emu {
        let mut emu = Emu {
            objects: arr![Object::open(); 32],
            baskets: arr![Basket::empty(); 128],
        };
        let mut basket = Basket::start(0, 0);
        basket.kids.insert(Loc::Phi, Kid::Requested);
        emu.baskets[0] = basket;
        emu
    }

    /// Add an additional object
    pub fn put(&mut self, ob: Ob, obj: Object) -> &mut Emu {
        assert!(
            self.objects[ob].is_empty(),
            "The object ν{} already occupied",
            ob
        );
        self.objects[ob] = obj;
        self
    }

    /// Make new basket for this attribute.
    pub fn new(&mut self, bk: Bk, loc: Loc) {
        if let Some(Kid::Requested) = self.basket(bk).kids.get(&loc) {
            let ob = self.basket(bk).ob;
            let obj = self.object(ob);
            if let Some((locator, advice)) = obj.attrs.get(&loc) {
                let (tob, psi) = self
                    .find(bk, locator)
                    .expect(&format!("Can't find {} from β{}/ν{}", locator, bk, ob));
                let tpsi = if *advice { bk } else { psi };
                let nbk = if let Some(ebk) = self.find_existing_data(tob) {
                    trace!(
                        "new(β{}/ν{}, {}) -> link to β{} since there is ν{}.Δ",
                        bk,
                        ob,
                        loc,
                        ebk,
                        tob
                    );
                    ebk
                } else if let Some(ebk) = self.find_existing(tob, tpsi) {
                    trace!(
                        "new(β{}/ν{}, {}) -> link to β{} since it's ν{}.β{}",
                        bk,
                        ob,
                        loc,
                        ebk,
                        tob,
                        tpsi
                    );
                    ebk
                } else {
                    let id = self
                        .baskets
                        .iter()
                        .find_position(|b| b.is_empty())
                        .unwrap()
                        .0 as Bk;
                    let mut bsk = Basket::start(tob, tpsi);
                    for k in self.object(tob).attrs.keys() {
                        bsk.kids.insert(k.clone(), Kid::Empty);
                    }
                    bsk.kids.insert(Loc::Phi, Kid::Requested);
                    self.baskets[id as usize] = bsk;
                    trace!("new(β{}/ν{}, {}) -> β{}", bk, ob, loc, id);
                    id
                };
                let _ = &self.baskets[bk as usize]
                    .kids
                    .insert(loc.clone(), Kid::Waiting(nbk));
            }
        }
    }

    /// Give control to the atom of the basket.
    pub fn delegate(&mut self, bk: Bk) {
        if let Some(Kid::Requested) = self.basket(bk).kids.get(&Loc::Phi) {
            let bsk = self.basket(bk);
            if bsk
                .kids
                .values()
                .find(|k| matches!(k, Kid::Waiting(_)))
                .is_none()
            {
                let obj = self.object(bsk.ob);
                if let Some(a) = obj.lambda {
                    if let Some(d) = a(self, bk) {
                        self.write(bk, Loc::Phi, d);
                        trace!("delegate(β{}) -> 0x{:04X})", bk, d);
                    }
                }
            }
        }
    }

    /// Copy data from object to basket.
    pub fn copy(&mut self, bk: Bk) {
        if let Some(Kid::Requested) = self.basket(bk).kids.get(&Loc::Phi) {
            let obj = self.object(self.basket(bk).ob);
            if let Some(d) = obj.delta {
                self.write(bk, Loc::Phi, d);
                trace!("copy(β{}) -> 0x{:04X}", bk, d);
            }
        }
    }

    /// Propagate the value from this attribute to the one expecting it.
    pub fn propagate(&mut self, bk: Bk) {
        let mut changes = vec![];
        if let Some(Kid::Dataized(d)) = self.basket(bk).kids.get(&Loc::Phi) {
            for i in 0..self.baskets.len() {
                let bsk = self.basket(i as Bk);
                for k in bsk.kids.keys() {
                    if let Some(Kid::Waiting(b)) = &bsk.kids.get(k) {
                        if *b == bk {
                            changes.push((i as Bk, k.clone(), *d));
                        }
                    }
                }
            }
        }
        for (b, l, d) in changes.iter() {
            let _ = &self.baskets[*b as usize]
                .kids
                .insert(l.clone(), Kid::Dataized(*d));
            trace!("propagate(β{}) : 0x{:04X} to β{}.{}", bk, *d, b, l);
        }
    }

    /// Delete the basket if it's already finished.
    pub fn delete(&mut self, bk: Bk) {
        if bk != ROOT_BK {
            if let Some(Kid::Dataized(_)) = self.basket(bk).kids.get(&Loc::Phi) {
                let mut waiting = false;
                for i in 0..self.baskets.len() {
                    let bsk = self.basket(i as Bk);
                    for k in bsk.kids.keys() {
                        if let Some(Kid::Waiting(b)) = &bsk.kids.get(k) {
                            if *b == bk {
                                waiting = true
                            }
                        }
                    }
                }
                if !waiting {
                    let obj = self.object(self.basket(bk).ob);
                    if !obj.constant {
                        self.baskets[bk as usize] = Basket::empty();
                        trace!("delete(β{})", bk);
                    }
                }
            }
        }
    }

    /// Request dataization of one attribute of the basket.
    pub fn request(&mut self, bk: Bk, loc: Loc) {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Empty) => {
                let _ = &self.baskets[bk as usize]
                    .kids
                    .insert(loc.clone(), Kid::Requested);
                trace!("request(β{}, {}): requested", bk, loc);
            }
            Some(k) => panic!(
                "Can't request {} in β{} since it's already {}:\n{}",
                loc, bk, k, self
            ),
        };
    }

    /// Read data if available.
    pub fn read(&mut self, bk: Bk, loc: Loc) -> Option<Data> {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Empty) => {
                self.request(bk, loc);
                None
            }
            Some(Kid::Requested) => None,
            Some(Kid::Waiting(_)) => None,
            Some(Kid::Dataized(d)) => Some(*d),
        }
    }

    /// Write data into the attribute of the box.
    fn write(&mut self, bk: Bk, loc: Loc, d: Data) {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Requested) | Some(Kid::Waiting(_)) => {
                let _ = &self.baskets[bk as usize]
                    .kids
                    .insert(loc.clone(), Kid::Dataized(d));
                trace!("write(β{}, {}, 0x{:04X})", bk, loc, d);
            }
            Some(k) => panic!(
                "Can't save 0x{:04X} to {} in β{} since it's {}:\n{}",
                d, loc, bk, k, self
            ),
        };
    }

    /// Suppose, the incoming locator is `^.0.@.2`. We have to find the right
    /// object in the catalog of them and return the position of the found one
    /// together with the suggested \psi.
    fn find(&self, bk: Bk, locator: &Locator) -> Result<(Ob, Bk), String> {
        let mut bsk = self.basket(bk);
        let mut locs = locator.to_vec();
        let mut ret = Err("Nothing found".to_string());
        let mut ob = 0;
        let mut log = vec![];
        let mut psi: Bk = bsk.psi;
        ret = loop {
            if locs.is_empty() {
                break ret;
            }
            let loc = locs.remove(0);
            log.push(loc.to_string());
            let next = match loc {
                Loc::Root => ROOT_OB,
                Loc::Psi => {
                    if bsk.psi == ROOT_BK {
                        return Err(format!("The root doesn't have 𝜉: {}", join!(log)));
                    }
                    psi = bsk.psi;
                    bsk = self.basket(psi);
                    log.push(format!("𝜉=β{}/ν{}", psi, bsk.ob));
                    bsk.ob
                }
                Loc::Obj(i) => i as Ob,
                _ => match self.object(ob).attrs.get(&loc) {
                    None => match self.object(ob).attrs.get(&Loc::Phi) {
                        None => {
                            return Err(format!(
                                "Can't find {} in ν{} and there is no φ: {}",
                                loc,
                                ob,
                                join!(log)
                            ))
                        }
                        Some((p, _psi)) => {
                            locs.insert(0, loc);
                            locs.splice(0..0, p.to_vec());
                            log.push(format!("++{}", p));
                            ob
                        }
                    },
                    Some((p, _psi)) => {
                        locs.splice(0..0, p.to_vec());
                        log.push(format!("+{}", p));
                        ob
                    }
                },
            };
            ob = next;
            ret = Ok((next, psi))
        };
        if let Ok((next, _psi)) = ret {
            if self.object(next).is_empty() {
                return Err(format!(
                    "The object ν{} is found by β{}.{}, but it's empty",
                    next, bk, locator
                ));
            }
        }
        trace!(
            "find(β{}/ν{}, {}) -> (ν{}, β{}) : {}",
            bk,
            self.basket(bk).ob,
            locator,
            ret.clone().unwrap().0,
            ret.clone().unwrap().1,
            join!(log)
        );
        ret
    }

    /// Find already existing basket.
    fn find_existing(&self, ob: Ob, psi: Bk) -> Option<Bk> {
        let found = self
            .baskets
            .iter()
            .find_position(|b| b.ob == ob && b.psi == psi);
        match found {
            Some((pos, _bsk)) => Some(pos as Bk),
            None => None,
        }
    }

    /// Find already existing basket pointing to the object with data.
    fn find_existing_data(&self, ob: Ob) -> Option<Bk> {
        let found = self
            .baskets
            .iter()
            .find_position(|bsk| bsk.ob == ob && self.object(bsk.ob).delta.is_some());
        match found {
            Some((pos, _bsk)) => Some(pos as Bk),
            None => None,
        }
    }

    /// Run a single transition cycle.
    pub fn cycle(&mut self) -> (Data, usize) {
        let mut cycles: usize = 1;
        loop {
            self.cycle_one();
            if let Some(Kid::Dataized(d)) = self.basket(ROOT_BK).kids.get(&Loc::Phi) {
                trace!("cycle() -> 0x{:04X} in #{} cycle(s)", *d, cycles);
                return (*d, cycles);
            }
            cycles += 1;
            if cycles > 1000 {
                panic!(
                    "Too many cycles ({}), most probably endless recursion:\n{}",
                    cycles, self
                );
            }
        }
    }

    fn cycle_one(&mut self) {
        for i in 0..self.baskets.len() {
            let bk = i as Bk;
            self.copy(bk);
            self.delegate(bk);
            self.delete(bk);
            self.propagate(bk);
            for loc in self.locs(bk) {
                self.new(bk, loc.clone());
            }
        }
    }

    /// Take all locs from the given basket.
    fn locs(&self, bk: Bk) -> Vec<Loc> {
        let mut keys = vec![];
        for (k, _) in &self.basket(bk).kids {
            keys.push(k.clone());
        }
        keys
    }

    fn object(&self, ob: Ob) -> &Object {
        &self.objects[ob]
    }

    fn basket(&self, bk: Bk) -> &Basket {
        &self.baskets[bk as usize]
    }
}

#[cfg(test)]
use crate::ph;

#[test]
pub fn simple_dataization_cycle() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v1"), true));
    emu.put(1, Object::dataic(42));
    assert_eq!(42, emu.cycle().0);
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v2"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("v1"), false));
    assert_eq!(42, emu.cycle().0);
}

#[test]
pub fn with_many_decorators() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v4"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("v1"), false));
    emu.put(3, Object::open().with(Loc::Phi, ph!("v2"), false));
    emu.put(4, Object::open().with(Loc::Phi, ph!("v3"), false));
    assert_eq!(42, emu.cycle().0);
}

// []
//   42 > x
//   42 > y
//   int-add > @
//     $.x
//     $.y
#[test]
pub fn summarizes_two_numbers() {
    assert_emu!(
        84,
        "
        ν0 ↦ ⟦ φ ↦ ν3 ⟧
        ν1 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2 ↦ ⟦ λ ↦ int-add, ρ ↦ 𝜉.𝛼0, 𝛼0 ↦ 𝜉.𝛼1 ⟧
        ν3 ↦ ⟦ φ ↦ ν2(𝜉), 𝛼0 ↦ ν1, 𝛼1 ↦ ν1 ⟧
        ν5 ↦ ⟦ φ ↦ ν3(𝜉) ⟧
        "
    );
}

// []
//   int-add > x!          v1
//     2                   v2
//     3                   v3
//   int-add > @           v4
//     x
//     x
#[test]
pub fn summarizes_two_pairs_of_numbers() {
    assert_emu!(
        10,
        "
        ν0 ↦ ⟦ φ ↦ ν4 ⟧
        ν1 ↦ ⟦ λ ↦ int-add, ρ ↦ ν2, 𝛼0 ↦ ν3 ⟧
        ν2 ↦ ⟦ Δ ↦ 0x0002 ⟧
        ν3 ↦ ⟦ Δ ↦ 0x0003 ⟧
        ν4 ↦ ⟦ λ ↦ int-add, ρ ↦ ν1, 𝛼0 ↦ ν1 ⟧
        "
    );
}

// [x] > a
//   $.x > @
// a > foo
//   a 42 > @
#[test]
pub fn calls_itself_once() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν4 ⟧
        ν1 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν2 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν3 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν2 ⟧
        ν4 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν3 ⟧
        "
    );
}

// [x] > a
//   $.x > @
// [y] > b
//   a > @
//     $.y
// b 42 > foo
#[test]
pub fn injects_xi_correctly() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν5 ⟧
        ν1 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν2 ↦ ⟦ φ ↦ ν3(𝜉) ⟧
        ν3 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ 𝜉.𝛼0 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν5 ↦ ⟦ φ ↦ ν2(𝜉), 𝛼0 ↦ ν4 ⟧
        "
    );
}

// [a3] > v1         v1
//   $.a3 > @
// [a1] > v2         v2
//   v1 > @          v3
//     $.a1
// v2 42 > @         v4
#[test]
pub fn reverse_to_abstract() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν3 ⟧
        ν1 ↦ ⟦ φ ↦ 𝜉.𝛼3 ⟧
        ν2 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼3 ↦ 𝜉.𝛼1 ⟧
        ν3 ↦ ⟦ φ ↦ ν2(𝜉), 𝛼1 ↦ ν4 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a          v1
//   b > @          v2
//     c            v3
//       $.x
// [x] > b          v4
//   x > @
// [x] > c          v5
//   x > @
// a                v6
//   42             v7
#[test]
pub fn passes_xi_through_two_layers() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν6 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(𝜉), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν5(𝜉), 𝛼0 ↦ 𝜉.𝜉.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν5 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν7 ⟧
        ν7 ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a          v1
//   b > @          v2
//     c            v3
//       d          v4
//         $.x
// [x] > b          v5
//   x > @
// [x] > c          v6
//   x > @
// [x] > d          v7
//   x > @
// a                v8
//   42             v9
#[test]
pub fn passes_xi_through_three_layers() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν8 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν5(𝜉), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν6(𝜉), 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ φ ↦ ν7(𝜉), 𝛼0 ↦ 𝜉.𝜉.𝜉.𝛼0 ⟧
        ν5 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν8 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν9 ⟧
        ν9 ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a          v1
//   b > @          v2
//     c            v3
//       d          v4
//         e        v5
//           $.x
// [x] > b          v6
//   x > @
// [x] > c          v7
//   x > @
// [x] > d          v8
//   x > @
// [x] > e          v9
//   x > @
// a                v10
//   42             v11
#[test]
pub fn passes_xi_through_four_layers() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν10 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν6(𝜉), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν7(𝜉), 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ φ ↦ ν8(𝜉), 𝛼0 ↦ ν5 ⟧
        ν5 ↦ ⟦ φ ↦ ν9(𝜉), 𝛼0 ↦ 𝜉.𝜉.𝜉.𝜉.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν8 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν9 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν10 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν11 ⟧
        ν11 ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a        v1
//   b > @        v2
//     c          v3
//       $.x
// [x] > b        v4
//   c > @        v5
//     $.x
// [x] > c        v6
//   x > @
// a              v7
//   42           v8
#[test]
pub fn simulation_of_recursion() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν7 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(𝜉), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν6(𝜉), 𝛼0 ↦ 𝜉.𝜉.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ ν5 ⟧
        ν5 ↦ ⟦ φ ↦ ν6(𝜉), 𝛼0 ↦ 𝜉.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν8 ⟧
        ν8 ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > a        v1
//   b > @        v2
//     f          v3
//       $.x
// [x] > b        v4
//   c > @        v5
//     f          v6
//       $.x
// [x] > c        v7
//   f > @        v8
//     $.x
// [x] > f        v9
//   x > @
// a              v10
//   42           v11
#[test]
pub fn deep_simulation_of_recursion() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν10 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(𝜉), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν9(𝜉), 𝛼0 ↦ 𝜉.𝜉.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ ν5 ⟧
        ν5 ↦ ⟦ φ ↦ ν7(𝜉), 𝛼0 ↦ ν6 ⟧
        ν6 ↦ ⟦ φ ↦ ν9(𝜉), 𝛼0 ↦ 𝜉.𝜉.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ ν8 ⟧
        ν8 ↦ ⟦ φ ↦ ν9(𝜉), 𝛼0 ↦ 𝜉.𝛼0 ⟧
        ν9 ↦ ⟦ φ ↦ 𝜉.𝛼0 ⟧
        ν10 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν11 ⟧
        ν11 ↦ ⟦ Δ ↦ 0x002A ⟧
        "
    );
}

// [x] > foo        v1
//   bool-if        v2
//     int-less     v3
//       $.x
//       0          v4
//     42           v5
//     foo          v6
//       int-sub    v7
//         $.x
//         1        v8
// foo              v9
//   7              v10
#[test]
pub fn simple_recursion() {
    assert_emu!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν9 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ λ ↦ bool-if, ρ ↦ ν3, 𝛼0 ↦ ν5, 𝛼1 ↦ ν6 ⟧
        ν3 ↦ ⟦ λ ↦ int-less, ρ ↦ 𝜉.𝛼0, 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x0000 ⟧
        ν5 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν6 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν7 ⟧
        ν7 ↦ ⟦! λ ↦ int-sub, ρ ↦ 𝜉.𝜉.𝛼0, 𝛼0 ↦ ν8 ⟧
        ν8 ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν9 ↦ ⟦ φ ↦ ν1(𝜉), 𝛼0 ↦ ν10 ⟧
        ν10 ↦ ⟦ Δ ↦ 0x0007 ⟧
    "
    );
}

#[test]
pub fn recursive_fibonacci() {
    assert_emu!(
        21,
        "
        ν0 ↦ ⟦ φ ↦ ν2 ⟧
        ν1 ↦ ⟦ Δ ↦ 0x0007 ⟧
        ν2 ↦ ⟦ φ ↦ ν3(𝜉), 𝛼0 ↦ ν1 ⟧
        ν3 ↦ ⟦ φ ↦ ν13 ⟧
        ν5 ↦ ⟦ Δ ↦ 0x0002 ⟧
        ν6 ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜉.𝜉.𝛼0, 𝛼0 ↦ ν5 ⟧
        ν7 ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν8 ↦ ⟦ λ ↦ int-sub, ρ ↦ 𝜉.𝜉.𝛼0, 𝛼0 ↦ ν7 ⟧
        ν9 ↦ ⟦ φ ↦ ν3(𝜉), 𝛼0 ↦ ν8 ⟧
        ν10 ↦ ⟦ φ ↦ ν3(𝜉), 𝛼0 ↦ ν6 ⟧
        ν11 ↦ ⟦ λ ↦ int-add, ρ ↦ ν9, 𝛼0 ↦ ν10 ⟧
        ν12 ↦ ⟦ λ ↦ int-less, ρ ↦ 𝜉.𝛼0, 𝛼0 ↦ ν5 ⟧
        ν13 ↦ ⟦ λ ↦ bool-if, ρ ↦ ν12, 𝛼0 ↦ ν7, 𝛼1 ↦ ν11 ⟧
        "
    );
}
