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
use crate::perf::{Perf, Transition};
use arr_macro::arr;
use itertools::Itertools;
use log::{debug, trace};
use regex::Regex;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::time::Instant;

pub const ROOT_BK: Bk = 0;
pub const ROOT_OB: Ob = 0;

const MAX_CYCLES: usize = 1000;
const MAX_OBJECTS: usize = 32;
const MAX_BASKETS: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opt {
    DontDelete,
    LogSnapshots,
    StopWhenTooManyCycles,
    StopWhenStuck,
}

pub struct Emu {
    pub objects: [Object; MAX_OBJECTS],
    pub baskets: [Basket; MAX_BASKETS],
    pub opts: HashSet<Opt>,
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
macro_rules! assert_dataized_eq {
    ($eq:expr, $txt:expr) => {
        let mut emu: Emu = $txt.parse().unwrap();
        emu.opt(Opt::DontDelete);
        emu.opt(Opt::StopWhenTooManyCycles);
        assert_eq!(
            $eq,
            emu.dataize().0,
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
            baskets: arr![Basket::empty(); 256],
            opts: HashSet::new(),
        };
        let mut basket = Basket::start(0, 0);
        basket.kids.insert(Loc::Phi, Kid::Rqtd);
        emu.baskets[0] = basket;
        emu
    }

    pub fn opt(&mut self, opt: Opt) {
        self.opts.insert(opt);
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

    /// Copy data from object to basket.
    pub fn copy(&mut self, perf: &mut Perf, bk: Bk) {
        let bsk = self.basket(bk);
        if let Some(Kid::Rqtd) = bsk.kids.get(&Loc::Phi) {
            let obj = self.object(bsk.ob);
            if let Some(d) = obj.delta {
                let _ = &self.baskets[bk as usize]
                    .kids
                    .insert(Loc::Phi, Kid::Dtzd(d));
                trace!("copy(β{}) -> 0x{:04X}", bk, d);
                perf.hit(Transition::CPY);
            }
        }
        perf.tick(Transition::CPY);
    }

    /// Propagate the value from this attribute to the one expecting it.
    pub fn propagate(&mut self, perf: &mut Perf, bk: Bk, loc: Loc) {
        let mut changes = vec![];
        if let Some(Kid::Dtzd(d)) = self.basket(bk).kids.get(&loc) {
            for i in 0..self.baskets.len() {
                let bsk = self.basket(i as Bk);
                if bsk.is_empty() {
                    continue;
                }
                for k in bsk.kids.keys() {
                    if let Some(Kid::Wait(b, l)) = &bsk.kids.get(k) {
                        if *b == bk && *l == loc {
                            changes.push((i as Bk, k.clone(), *d));
                        }
                    }
                    perf.tick(Transition::PPG);
                }
            }
        }
        for (b, l, d) in changes.iter() {
            let _ = &self.baskets[*b as usize]
                .kids
                .insert(l.clone(), Kid::Dtzd(*d));
            trace!("propagate(β{}, {}) : 0x{:04X} to β{}.{}", bk, loc, *d, b, l);
            perf.hit(Transition::PPG);
        }
        perf.tick(Transition::PPG);
    }

    /// Delete the basket if it's already finished.
    pub fn delete(&mut self, perf: &mut Perf, bk: Bk) {
        if bk != ROOT_BK {
            if let Some(Kid::Dtzd(_)) = self.basket(bk).kids.get(&Loc::Phi) {
                let mut waiting = false;
                for i in 0..self.baskets.len() {
                    let bsk = self.basket(i as Bk);
                    if bsk.is_empty() {
                        continue;
                    }
                    perf.tick(Transition::DEL);
                    for k in bsk.kids.keys() {
                        if let Some(Kid::Wait(b, _)) = &bsk.kids.get(k) {
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
                        perf.hit(Transition::DEL);
                    }
                }
            }
            perf.tick(Transition::DEL);
        }
    }

    /// Give control to the atom of the basket.
    pub fn delegate(&mut self, perf: &mut Perf, bk: Bk) {
        let bsk = self.basket(bk);
        if let Some(Kid::Rqtd) = bsk.kids.get(&Loc::Phi) {
            if bsk
                .kids
                .values()
                .find(|k| matches!(k, Kid::Wait(_, _)))
                .is_none()
            {
                let obj = self.object(bsk.ob);
                if let Some((n, func)) = &obj.lambda {
                    let name = n.clone();
                    if let Some(d) = func(self, bk) {
                        perf.atom(name);
                        let _ = &self.baskets[bk as usize]
                            .kids
                            .insert(Loc::Phi, Kid::Dtzd(d));
                        trace!("delegate(β{}) -> 0x{:04X}", bk, d);
                        perf.hit(Transition::DLG);
                    }
                }
            }
        }
        perf.tick(Transition::DLG);
    }

    /// Make new basket for this attribute.
    pub fn find(&mut self, perf: &mut Perf, bk: Bk, loc: Loc) {
        if let Some(Kid::Rqtd) = self.basket(bk).kids.get(&loc) {
            let ob = self.basket(bk).ob;
            let obj = self.object(ob);
            if let Some((locator, _)) = obj.attrs.get(&loc) {
                let (_tob, _psi, attr) = self
                    .search(bk, locator)
                    .expect(&format!("Can't find {} from β{}/ν{}", locator, bk, ob));
                if let Some((pbk, ploc)) = attr {
                    let bsk = self.basket(pbk);
                    if let Some(Kid::Empt) = bsk.kids.get(&ploc) {
                        let _ = &self.baskets[pbk as usize]
                            .kids
                            .insert(ploc.clone(), Kid::Wait(bk, loc.clone()));
                        let _ = &self.baskets[bk as usize]
                            .kids
                            .insert(loc.clone(), Kid::Need);
                    } else {
                        let _ = &self.baskets[bk as usize]
                            .kids
                            .insert(loc.clone(), Kid::Wait(pbk, ploc.clone()));
                    }
                } else {
                    let _ = &self.baskets[bk as usize]
                        .kids
                        .insert(loc.clone(), Kid::Need);
                }
                perf.hit(Transition::FND);
            }
        }
        perf.tick(Transition::FND);
    }

    /// Make new basket for this attribute.
    pub fn new(&mut self, perf: &mut Perf, bk: Bk, loc: Loc) {
        if let Some(Kid::Need) = self.basket(bk).kids.get(&loc) {
            let ob = self.basket(bk).ob;
            let obj = self.object(ob);
            if let Some((locator, advice)) = obj.attrs.get(&loc) {
                let (tob, psi, _attr) = self
                    .search(bk, locator)
                    .expect(&format!("Can't find {} from β{}/ν{}", locator, bk, ob));
                let tpsi = if *advice { bk } else { psi };
                let nbk = if let Some(ebk) = self.stashed(tob, tpsi) {
                    trace!("new(β{}/ν{}, {}) -> link to stashed β{}", bk, ob, loc, ebk);
                    ebk
                } else {
                    let id = self
                        .baskets
                        .iter()
                        .find_position(|b| b.is_empty())
                        .expect(
                            format!("No more empty baskets left in the pool of {}", MAX_BASKETS)
                                .as_str(),
                        )
                        .0 as Bk;
                    let mut bsk = Basket::start(tob, tpsi);
                    for k in self.object(tob).attrs.keys() {
                        bsk.kids.insert(k.clone(), Kid::Empt);
                    }
                    bsk.kids.insert(Loc::Phi, Kid::Rqtd);
                    self.baskets[id as usize] = bsk;
                    trace!("new(β{}/ν{}, {}) -> β{} created", bk, ob, loc, id);
                    id
                };
                perf.hit(Transition::NEW);
                let _ = &self.baskets[bk as usize]
                    .kids
                    .insert(loc.clone(), Kid::Wait(nbk, Loc::Phi));
            }
        }
        perf.tick(Transition::NEW);
    }

    /// Read data if available.
    pub fn read(&mut self, bk: Bk, loc: Loc) -> Option<Data> {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Empt) => {
                let _ = &self.baskets[bk as usize]
                    .kids
                    .insert(loc.clone(), Kid::Rqtd);
                trace!("read(β{}, {}): was empty, requested", bk, loc);
                None
            }
            Some(Kid::Need) | Some(Kid::Wait(_, _)) | Some(Kid::Rqtd) => None,
            Some(Kid::Dtzd(d)) => Some(*d),
        }
    }

    /// Suppose, the incoming locator is `^.0.@.2`. We have to find the right
    /// object in the catalog of them and return the position of the found one
    /// together with the suggested \psi.
    fn search(&self, bk: Bk, locator: &Locator) -> Result<(Ob, Bk, Option<(Bk, Loc)>), String> {
        let mut bsk = self.basket(bk);
        let mut attr = None;
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
                Loc::Xi => {
                    if bsk.psi == ROOT_BK {
                        return Err(format!("Object Φ doesn't have ξ: {}", join!(log)));
                    }
                    psi = bsk.psi;
                    attr = Some((psi, Loc::Root));
                    bsk = self.basket(psi);
                    log.push(format!("ξ=β{}/ν{}", psi, bsk.ob));
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
                            locs.insert(0, loc.clone());
                            attr = Some((attr.unwrap().0, loc));
                            locs.splice(0..0, p.to_vec());
                            log.push(format!("++{}", p));
                            ob
                        }
                    },
                    Some((p, _psi)) => {
                        attr = Some((attr.unwrap().0, loc.clone()));
                        locs.splice(0..0, p.to_vec());
                        log.push(format!("+{}", p));
                        ob
                    }
                },
            };
            ob = next;
            ret = Ok((next, psi, attr.clone()))
        };
        if let Ok((next, _psi, _attr)) = ret.clone() {
            if self.object(next).is_empty() {
                return Err(format!(
                    "Object ν{} is found by β{}.{}, but it's empty",
                    next, bk, locator
                ));
            }
        }
        trace!(
            "find(β{}/ν{}, {}) -> (ν{}, β{}) : {} {}",
            bk,
            self.basket(bk).ob,
            locator,
            ret.clone().unwrap().0,
            ret.clone().unwrap().1,
            join!(log),
            if let Some((bk, loc)) = ret.clone().unwrap().2 {
                format!("[β{}.{}]", bk, loc)
            } else {
                "".to_string()
            }
        );
        ret
    }

    /// Find already existing basket.
    fn stashed(&self, ob: Ob, psi: Bk) -> Option<Bk> {
        if let Some((pos, _bsk)) = self.baskets.iter().find_position(|bsk| {
            if bsk.ob != ob {
                return false;
            }
            let obj = self.object(bsk.ob);
            if obj.delta.is_some() {
                return true;
            }
            if !obj.constant {
                return false;
            }
            return bsk.psi == psi;
        }) {
            return Some(pos as Bk);
        }
        None
    }

    fn object(&self, ob: Ob) -> &Object {
        &self.objects[ob]
    }

    fn basket(&self, bk: Bk) -> &Basket {
        &self.baskets[bk as usize]
    }
}

impl Emu {
    /// Dataize the first object.
    pub fn dataize(&mut self) -> (Data, Perf) {
        let mut cycles = 0;
        let mut perf = Perf::new();
        let time = Instant::now();
        loop {
            let before = perf.total_hits();
            self.cycle(&mut perf);
            if self.opts.contains(&Opt::LogSnapshots) {
                debug!(
                    "dataize() +{} hits in cycle #{}:\n{}",
                    perf.total_hits() - before,
                    cycles,
                    self
                );
            }
            if self.opts.contains(&Opt::StopWhenStuck) && before == perf.total_hits() {
                panic!(
                    "We are stuck, no hits after {}, in the recent cycle #{}:\n{}",
                    perf.total_hits(),
                    cycles,
                    self
                );
            }
            perf.cycles += 1;
            if let Some(Kid::Dtzd(d)) = self.basket(ROOT_BK).kids.get(&Loc::Phi) {
                debug!(
                    "dataize() -> 0x{:04X} in {:?}\n{}\n{}",
                    *d,
                    time.elapsed(),
                    perf,
                    self
                );
                return (*d, perf);
            }
            cycles += 1;
            if self.opts.contains(&Opt::StopWhenTooManyCycles) && cycles > MAX_CYCLES {
                panic!(
                    "Too many cycles ({}), most probably endless recursion:\n{}",
                    cycles, self
                );
            }
        }
    }

    fn cycle(&mut self, perf: &mut Perf) {
        self.cycle_one(perf, |s, p, bk| s.copy(p, bk));
        self.cycle_one(perf, |s, p, bk| s.delegate(p, bk));
        if !self.opts.contains(&Opt::DontDelete) {
            self.cycle_one(perf, |s, p, bk| s.delete(p, bk));
        }
        self.cycle_one(perf, |s, p, bk| {
            for loc in s.locs(bk) {
                s.propagate(p, bk, loc.clone());
                s.find(p, bk, loc.clone());
                s.new(p, bk, loc);
            }
        });
    }

    fn cycle_one(&mut self, perf: &mut Perf, f: fn(&mut Emu, &mut Perf, Bk)) {
        for i in 0..self.baskets.len() {
            let bk = i as Bk;
            if self.basket(bk).is_empty() {
                continue;
            }
            f(self, perf, bk);
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
}

#[cfg(test)]
use crate::ph;

#[test]
pub fn simple_dataization_cycle() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v1"), true));
    emu.put(1, Object::dataic(42));
    assert_eq!(42, emu.dataize().0);
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v2"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("v1"), false));
    assert_eq!(42, emu.dataize().0);
}

#[test]
pub fn with_many_decorators() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v4"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("v1"), false));
    emu.put(3, Object::open().with(Loc::Phi, ph!("v2"), false));
    emu.put(4, Object::open().with(Loc::Phi, ph!("v3"), false));
    assert_eq!(42, emu.dataize().0);
}

// []
//   42 > x
//   42 > y
//   int-add > @
//     $.x
//     $.y
#[test]
pub fn summarizes_two_numbers() {
    assert_dataized_eq!(
        84,
        "
        ν0 ↦ ⟦ φ ↦ ν3 ⟧
        ν1 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2 ↦ ⟦ λ ↦ int-add, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ξ.𝛼1 ⟧
        ν3 ↦ ⟦ φ ↦ ν2(ξ), 𝛼0 ↦ ν1, 𝛼1 ↦ ν1 ⟧
        ν5 ↦ ⟦ φ ↦ ν3(ξ) ⟧
        "
    );
}

// []
//   int-add > @    v1
//     int-add      v2
//       42         v9
//       42         v9
//     int-add      v3
//       int-neg    v4
//         42       v9
//       42         v9
//       42         v9
#[test]
pub fn preserves_calculation_results() {
    let mut emu = Emu::from_str(
        "
        ν0 ↦ ⟦ φ ↦ ν1 ⟧
        ν1 ↦ ⟦ λ ↦ int-add, ρ ↦ ν2, 𝛼0 ↦ ν3 ⟧
        ν2 ↦ ⟦ λ ↦ int-add, ρ ↦ ν9, 𝛼0 ↦ ν9 ⟧
        ν3 ↦ ⟦ λ ↦ int-add, ρ ↦ ν4, 𝛼0 ↦ ν9 ⟧
        ν4 ↦ ⟦ λ ↦ int-neg, ρ ↦ ν9 ⟧
        ν9 ↦ ⟦ Δ ↦ 0x002A ⟧
        ",
    )
    .unwrap();
    let dtz = emu.dataize();
    assert_eq!(84, dtz.0);
    let perf = dtz.1;
    assert_eq!(4, perf.total_atoms());
}

// []
//   foo > @        v1
//     int-add      v2
//       42         v9
//       42         v9
// [x] > foo        v3
//   int-add        v4
//     $.x
//     42           v9
#[test]
pub fn calculates_argument_once() {
    let mut emu = Emu::from_str(
        "
        ν0 ↦ ⟦ φ ↦ ν1 ⟧
        ν1 ↦ ⟦ λ ↦ int-add, ρ ↦ ν2, 𝛼0 ↦ ν3 ⟧
        ν2 ↦ ⟦ λ ↦ int-add, ρ ↦ ν9, 𝛼0 ↦ ν9 ⟧
        ν3 ↦ ⟦ λ ↦ int-add, ρ ↦ ν4, 𝛼0 ↦ ν9 ⟧
        ν4 ↦ ⟦ λ ↦ int-neg, ρ ↦ ν9 ⟧
        ν9 ↦ ⟦ Δ ↦ 0x002A ⟧
        ",
    )
    .unwrap();
    let dtz = emu.dataize();
    assert_eq!(84, dtz.0);
    let perf = dtz.1;
    assert_eq!(4, perf.total_atoms());
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
    assert_dataized_eq!(
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν4 ⟧
        ν1 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν2 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν3 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν2 ⟧
        ν4 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν3 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν5 ⟧
        ν1 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν2 ↦ ⟦ φ ↦ ν3(ξ) ⟧
        ν3 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ξ.𝛼0 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν5 ↦ ⟦ φ ↦ ν2(ξ), 𝛼0 ↦ ν4 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν3 ⟧
        ν1 ↦ ⟦ φ ↦ ξ.𝛼3 ⟧
        ν2 ↦ ⟦ φ ↦ ν1(ξ), 𝛼3 ↦ ξ.𝛼1 ⟧
        ν3 ↦ ⟦ φ ↦ ν2(ξ), 𝛼1 ↦ ν4 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν6 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(ξ), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν5(ξ), 𝛼0 ↦ ξ.ξ.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν5 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν7 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν8 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν5(ξ), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν6(ξ), 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ φ ↦ ν7(ξ), 𝛼0 ↦ ξ.ξ.ξ.𝛼0 ⟧
        ν5 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν8 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν9 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν10 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν6(ξ), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν7(ξ), 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ φ ↦ ν8(ξ), 𝛼0 ↦ ν5 ⟧
        ν5 ↦ ⟦ φ ↦ ν9(ξ), 𝛼0 ↦ ξ.ξ.ξ.ξ.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν8 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν9 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν10 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν11 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν7 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(ξ), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν6(ξ), 𝛼0 ↦ ξ.ξ.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ ν5 ⟧
        ν5 ↦ ⟦ φ ↦ ν6(ξ), 𝛼0 ↦ ξ.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν8 ⟧
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
    assert_dataized_eq!(
        42,
        "
        ν0 ↦ ⟦ φ ↦ ν10 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(ξ), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν9(ξ), 𝛼0 ↦ ξ.ξ.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ ν5 ⟧
        ν5 ↦ ⟦ φ ↦ ν7(ξ), 𝛼0 ↦ ν6 ⟧
        ν6 ↦ ⟦ φ ↦ ν9(ξ), 𝛼0 ↦ ξ.ξ.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ ν8 ⟧
        ν8 ↦ ⟦ φ ↦ ν9(ξ), 𝛼0 ↦ ξ.𝛼0 ⟧
        ν9 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
        ν10 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν11 ⟧
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
    let mut emu = Emu::from_str(
        "
        ν0 ↦ ⟦ φ ↦ ν9 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ λ ↦ bool-if, ρ ↦ ν3, 𝛼0 ↦ ν5, 𝛼1 ↦ ν6 ⟧
        ν3 ↦ ⟦ λ ↦ int-less, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x0000 ⟧
        ν5 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν6 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν7 ⟧
        ν7 ↦ ⟦ λ ↦ int-sub, ρ ↦ ξ.ξ.𝛼0, 𝛼0 ↦ ν8 ⟧
        ν8 ↦ ⟦ Δ ↦ 0x0001 ⟧
        ν9 ↦ ⟦ φ ↦ ν1(ξ), 𝛼0 ↦ ν10 ⟧
        ν10 ↦ ⟦ Δ ↦ 0x0007 ⟧
        ",
    )
    .unwrap();
    emu.opt(Opt::DontDelete);
    let dtz = emu.dataize();
    let perf = dtz.1;
    assert_eq!(9, emu.baskets.iter().filter(|bsk| bsk.ob == 1).count());
    assert_eq!(4, *perf.hits.get(&Transition::CPY).unwrap());
}

#[cfg(test)]
fn fibo(n: Data) -> Data {
    if n < 2 {
        return 1;
    }
    fibo(n - 1) + fibo(n - 2)
}

#[cfg(test)]
fn fibo_ops(n: Data) -> usize {
    if n < 2 {
        return 2;
    }
    fibo_ops(n - 1) + fibo_ops(n - 2) + 5
}

#[test]
pub fn recursive_fibonacci() {
    let input = 7;
    let mut emu = Emu::from_str(
        format!(
            "
            ν0 ↦ ⟦ φ ↦ ν2 ⟧
            ν1 ↦ ⟦ Δ ↦ 0x{:04X} ⟧
            ν2 ↦ ⟦ φ ↦ ν3(ξ), 𝛼0 ↦ ν1 ⟧
            ν3 ↦ ⟦ φ ↦ ν13 ⟧
            ν5 ↦ ⟦ Δ ↦ 0x0002 ⟧
            ν6 ↦ ⟦ λ ↦ int-sub, ρ ↦ ξ.ξ.𝛼0, 𝛼0 ↦ ν5 ⟧
            ν7 ↦ ⟦ Δ ↦ 0x0001 ⟧
            ν8 ↦ ⟦ λ ↦ int-sub, ρ ↦ ξ.ξ.𝛼0, 𝛼0 ↦ ν7 ⟧
            ν9 ↦ ⟦ φ ↦ ν3(ξ), 𝛼0 ↦ ν8 ⟧
            ν10 ↦ ⟦ φ ↦ ν3(ξ), 𝛼0 ↦ ν6 ⟧
            ν11 ↦ ⟦ λ ↦ int-add, ρ ↦ ν9, 𝛼0 ↦ ν10 ⟧
            ν12 ↦ ⟦ λ ↦ int-less, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ν5 ⟧
            ν13 ↦ ⟦ λ ↦ bool-if, ρ ↦ ν12, 𝛼0 ↦ ν7, 𝛼1 ↦ ν11 ⟧
            ",
            input
        )
        .as_str(),
    )
    .unwrap();
    let dtz = emu.dataize();
    assert_eq!(fibo(input), dtz.0, "Wrong number calculated");
    let perf = dtz.1;
    assert_eq!(
        perf.total_atoms(),
        fibo_ops(input),
        "Too many atomic operations"
    );
}
