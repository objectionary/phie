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

use regex::Regex;
use crate::basket::{Bk, Basket, Kid};
use crate::data::Data;
use crate::loc::Loc;
use crate::object::{Ob, Object};
use crate::path::Path;
use crate::ph;
use arr_macro::arr;
use std::str::FromStr;
use std::fmt;
use log::trace;
use itertools::Itertools;

pub const ROOT_BK: Bk = 0;
pub const ROOT_OB : Ob = 0;

pub struct Emu {
    pub objects: [Object; 256],
    pub baskets: [Basket; 256],
}

macro_rules! join {
    ($log:expr) => {
        $log.iter().join("; ")
    };
}

#[macro_export]
macro_rules! assert_emu {
    ($eq:expr, $txt:expr) => {
        let mut emu = Emu::parse_phi($txt).unwrap();
        assert_eq!($eq, emu.cycle().unwrap());
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
                ob, obj,
                self.baskets.iter().enumerate()
                    .filter(|(_, d)| !d.is_empty() && d.ob as usize == ob)
                    .map(|(i, d)| format!("\n\t➞ β{} {}", i, d))
                    .collect::<Vec<String>>()
                    .join("")
            ));
        }
        f.write_str(lines.join("\n").as_str())
    }
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional objects.
    pub fn empty() -> Emu {
        let mut emu = Emu {
            objects: arr![Object::open(); 256],
            baskets: arr![Basket::empty(); 256],
        };
        let mut basket = Basket::start(0, 0);
        basket.kids.insert(Loc::Phi, Kid::Start);
        emu.baskets[0] = basket;
        emu
    }

    /// Add an additional object
    pub fn put(&mut self, ob: Ob, obj: Object) -> &mut Emu {
        assert!(self.objects[ob].is_empty(), "The object ν{} already occupied", ob);
        self.objects[ob] = obj;
        self
    }

    /// Request dataization of phi-pointed objects.
    pub fn decorate(&mut self, bk: Bk) {
        if self.basket(bk).kids.contains_key(&Loc::Phi) {
            if let Some(Kid::Start) = self.basket(bk).kids.get(&Loc::Phi) {
                self.request(bk, Loc::Phi);
                trace!("decorate(β{})", bk);
            }
        }
    }

    /// Make new basket for this attribute.
    pub fn new(&mut self, bk: Bk, loc: Loc) {
        if let Some(Kid::Requested) = self.basket(bk).kids.get(&loc) {
            let nbk = self.baskets.iter().find_position(|b| b.is_empty()).unwrap().0 as Bk;
            let ob = self.basket(bk).ob;
            let obj = self.object(ob);
            if let Some((path, advice)) = obj.attrs.get(&loc) {
                let (target, psi) = self.find(bk, path).expect(
                    &format!("Can't find {} from β{}/ν{}", path, bk, ob)
                );
                let mut bsk = Basket::start(target, if *advice { bk } else { psi });
                for k in self.object(target).attrs.keys() {
                    bsk.kids.insert(k.clone(), Kid::Start);
                }
                bsk.kids.insert(Loc::Phi, Kid::Start);
                self.baskets[nbk as usize] = bsk;
                let _ = &self.baskets[bk as usize].kids.insert(loc.clone(), Kid::Waiting(nbk, Loc::Phi));
                self.request(nbk, Loc::Phi);
                trace!("new(β{}/ν{}, {}) -> β{}", bk, ob, loc, nbk);
            }
        }
    }

    /// Give control to the atom of the basket.
    pub fn delegate(&mut self, bk: Bk) {
        if let Some(Kid::Requested) = self.basket(bk).kids.get(&Loc::Phi) {
            let obj = self.object(self.basket(bk).ob);
            if let Some(a) = obj.lambda {
                if let Some(d) = a(self, bk) {
                    self.write(bk, Loc::Phi, d);
                    trace!("delegate(β{}) -> 0x{:04X})", bk, d);
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
    pub fn propagate(&mut self, bk: Bk, loc: Loc) {
        let mut changes = vec![];
        if let Some(Kid::Dataized(d)) = self.basket(bk).kids.get(&loc) {
            for i in 0..self.baskets.len() {
                let bsk = self.basket(i as Bk);
                for k in bsk.kids.keys() {
                    if let Some(Kid::Waiting(b, l)) = &bsk.kids.get(k) {
                        if *b == bk && *l == loc {
                            changes.push((i as Bk, k.clone(), *d));
                        }
                    }
                }
            }
        }
        if !changes.is_empty() {
            let _ = &self.baskets[bk as usize].kids.insert(loc.clone(), Kid::Propagated);
            for (b, l, d) in changes.iter() {
                let _ = &self.baskets[*b as usize].kids.insert(l.clone(), Kid::Dataized(*d));
                trace!("propagate(β{}, {}) : 0x{:04X} to β{}.{}", bk, loc, *d, b, l);
            }
        }
    }

    /// Delete the basket if it's already finished.
    pub fn delete(&mut self, bk: Bk) {
        if bk != ROOT_BK {
            if let Some(Kid::Propagated) = self.basket(bk).kids.get(&Loc::Phi) {
                self.baskets[bk as usize] = Basket::empty();
                trace!("delete(β{})", bk);
            }
        }
    }

    /// Request dataization of one attribute of the basket.
    pub fn request(&mut self, bk: Bk, loc: Loc) {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Start) => {
                let _ = &self.baskets[bk as usize].kids.insert(loc.clone(), Kid::Requested);
                trace!("request(β{}, {}): requested", bk, loc);
            },
            Some(k) => panic!("Can't request {} in β{} since it's already {}:\n{}", loc, bk, k, self),
        };
    }

    /// Write data into the attribute of the box.
    pub fn write(&mut self, bk: Bk, loc: Loc, d: Data) {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Requested) | Some(Kid::Waiting(_, _)) => {
                let _ = &self.baskets[bk as usize].kids.insert(loc.clone(), Kid::Dataized(d));
                trace!("write(β{}, {}, 0x{:04X})", bk, loc, d);
            },
            Some(k) => panic!("Can't save 0x{:04X} to {} in β{} since it's {}:\n{}", d, loc, bk, k, self),
        };
    }

    /// Read data if available.
    pub fn read(&mut self, bk: Bk, loc: Loc) -> Option<Data> {
        match self.basket(bk).kids.get(&loc) {
            None => panic!("Can't find {} in β{}:\n{}", loc, bk, self),
            Some(Kid::Start) => {
                self.request(bk, loc);
                None
            },
            Some(Kid::Requested) => None,
            Some(Kid::Waiting(_, _)) => None,
            Some(Kid::Dataized(d)) => {
                trace!("read(β{}, {}) -> 0x{:04X}", bk, loc, *d);
                Some(*d)
            },
            Some(Kid::Propagated) => None,
        }
    }

    pub fn parse_phi(txt: &str) -> Result<Emu, String> {
        let mut emu = Emu::empty();
        let re_line = Regex::new("ν(\\d+) ↦ (⟦.*⟧)").unwrap();
        for line in txt.trim().split("\n").map(|t| t.trim()) {
            let caps = re_line.captures(line).unwrap();
            let v : Ob = caps.get(1).unwrap().as_str().parse().unwrap();
            emu.put(v, Object::from_str(caps.get(2).unwrap().as_str()).unwrap());
        }
        Ok(emu)
    }

    /// Suppose, the incoming path is `^.0.@.2`. We have to find the right
    /// object in the catalog of them and return the position of the found one
    /// together with the suggested \psi.
    fn find(&self, bk: Bk, path: &Path) -> Result<(Ob, Bk), String> {
        let mut bsk = self.basket(bk);
        let mut locs = path.to_vec();
        let mut ret = Err("Nothing found".to_string());
        let mut last = 0;
        let mut obj : &Object = self.object(bsk.ob);
        let mut log = vec![];
        let mut psi : Bk = bsk.psi;
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
                        return Err(format!("The root doesn't have 𝜓: {}", join!(log)))
                    }
                    psi = bsk.psi;
                    bsk = self.basket(psi);
                    let ob = bsk.ob;
                    log.push(format!("𝜓=β{}/ν{}", psi, ob));
                    ob
                },
                Loc::Obj(i) => i as Ob,
                Loc::Attr(_) => match obj.attrs.get(&loc) {
                    None => match obj.attrs.get(&Loc::Phi) {
                        None => return Err(format!("Can't find {} in ν{} and there is no φ: {}", loc, last, join!(log))),
                        Some((p, _psi)) => {
                            locs.insert(0, loc);
                            locs.splice(0..0, p.to_vec());
                            log.push(format!("+{}", p));
                            last
                        }
                    },
                    Some((p, _psi)) => {
                        locs.splice(0..0, p.to_vec());
                        log.push(format!("+{}", p));
                        last
                    }
                },
                _ => match obj.attrs.get(&loc) {
                    None => return Err(format!("Can't get {} from ν{}: {}", loc, last, join!(log))),
                    Some((p, _psi)) => {
                        locs.splice(0..0, p.to_vec());
                        log.push(format!("+{}", p));
                        last
                    }
                },
            };
            obj = self.object(next);
            last = next;
            ret = Ok((next, psi))
        };
        trace!(
            "find(β{}/ν{}, {}) -> (ν{}, β{}) : {}",
            bk, self.basket(bk).ob, path,
            ret.clone().unwrap().0, ret.clone().unwrap().1,
            join!(log)
        );
        ret
    }

    pub fn cycle(&mut self) -> Option<Data> {
        let mut cycles = 1;
        loop {
            trace!("Cycle #{}...", cycles);
            self.cycle_one();
            trace!("Emu:\n{}", self);
            if let Some(Kid::Dataized(d)) = self.basket(ROOT_BK).kids.get(&Loc::Phi) {
                return Some(*d);
            }
            cycles += 1;
            if cycles > 100 {
                panic!("Endless cycling:\n{}", self);
            }
        }
    }

    fn cycle_one(&mut self) {
        for i in 0..self.baskets.len() {
            let bk = i as Bk;
            self.copy(bk);
            self.decorate(bk);
            self.delegate(bk as Bk);
            self.delete(bk as Bk);
            for loc in self.keys(bk) {
                self.new(bk, loc.clone());
                self.propagate(bk, loc.clone());
            }
        }
    }

    pub fn keys(&self, bk: Bk) -> Vec<Loc> {
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

#[test]
pub fn simple_dataization_cycle() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v1"), true));
    emu.put(1, Object::dataic(42));
    assert_eq!(42, emu.cycle().unwrap());
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v2"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("v1"), false));
    assert_eq!(42, emu.cycle().unwrap());
}

#[test]
pub fn with_many_decorators() {
    let mut emu = Emu::empty();
    emu.put(0, Object::open().with(Loc::Phi, ph!("v4"), true));
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Loc::Phi, ph!("v1"), false));
    emu.put(3, Object::open().with(Loc::Phi, ph!("v2"), false));
    emu.put(4, Object::open().with(Loc::Phi, ph!("v3"), false));
    assert_eq!(42, emu.cycle().unwrap());
}

// []
//   42 > x
//   42 > y
//   int.add > @
//     $.x
//     $.y
#[test]
pub fn summarizes_two_numbers() {
    assert_emu!(84, "
        ν0 ↦ ⟦ φ ↦ ν3 ⟧
        ν1 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν2 ↦ ⟦ λ ↦ int.add, ρ ↦ 𝜓.𝛼0, 𝛼0 ↦ 𝜓.𝛼1 ⟧
        ν3 ↦ ⟦ φ ↦ ν2(𝜓), 𝛼0 ↦ ν1, 𝛼1 ↦ ν1 ⟧
        ν5 ↦ ⟦ φ ↦ ν3(𝜓) ⟧
    ");
}

// [x] > a
//   $.x > @
// a > foo
//   a 42 > @
#[test]
pub fn calls_itself_once() {
    assert_emu!(42, "
        ν0 ↦ ⟦ φ ↦ ν4 ⟧
        ν1 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν2 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν3 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν2 ⟧
        ν4 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν3 ⟧
    ");
}

// [x] > a
//   $.x > @
// [y] > b
//   a > @
//     $.y
// b 42 > foo
#[test]
pub fn injects_xi_correctly() {
    assert_emu!(42, "
        ν0 ↦ ⟦ φ ↦ ν5 ⟧
        ν1 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν2 ↦ ⟦ φ ↦ ν3(𝜓) ⟧
        ν3 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ 𝜓.𝛼0 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x002A ⟧
        ν5 ↦ ⟦ φ ↦ ν2(𝜓), 𝛼0 ↦ ν4 ⟧
    ");
}

// [a3] > v1         v1
//   $.a3 > @
// [a1] > v2         v2
//   v1 > @          v3
//     $.a1
// v2 42 > @         v4
#[test]
pub fn reverse_to_abstract() {
    assert_emu!(42, "
        ν0 ↦ ⟦ φ ↦ ν3 ⟧
        ν1 ↦ ⟦ φ ↦ 𝜓.𝛼3 ⟧
        ν2 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼3 ↦ 𝜓.𝛼1 ⟧
        ν3 ↦ ⟦ φ ↦ ν2(𝜓), 𝛼1 ↦ ν4 ⟧
        ν4 ↦ ⟦ Δ ↦ 0x002A ⟧
    ");
}

// [x] > a          v1  $=v6
//   b > @          v2  $=v6
//     c            v3  $=v2   v3 -> v6
//       $.x
// [x] > b          v4  $=v2
//   x > @
// [x] > c          v5  $=v3
//   x > @
// a                v6  $=R
//   42             v7
#[test]
pub fn passes_psi_through_two_layers() {
    assert_emu!(42, "
        ν0 ↦ ⟦ φ ↦ ν6 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(𝜓), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν5(𝜓), 𝛼0 ↦ 𝜓.𝜓.𝛼0 ⟧
        ν4 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν5 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν7 ⟧
        ν7 ↦ ⟦ Δ ↦ 0x002A ⟧
    ");
}

// [x] > a          v1  $=v8
//   b > @          v2  $=v8
//     c            v3  $=v2
//       d          v4  $=v3 -> v8
//         $.x
// [x] > b          v5  $=v2
//   x > @
// [x] > c          v6  $=v3
//   x > @
// [x] > d          v7  $=v4
//   x > @
// a                v8  $=R
//   42             v9
#[test]
pub fn passes_xi_through_three_layers() {
    assert_emu!(42, "
        ν0 ↦ ⟦ φ ↦ ν8 ⟧
        ν1 ↦ ⟦ φ ↦ ν2 ⟧
        ν2 ↦ ⟦ φ ↦ ν4(𝜓), 𝛼0 ↦ ν3 ⟧
        ν3 ↦ ⟦ φ ↦ ν5(𝜓), 𝛼0 ↦ ν4 ⟧
        ν4 ↦ ⟦ φ ↦ ν6(𝜓), 𝛼0 ↦ 𝜓.𝜓.𝛼0 ⟧
        ν5 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν6 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν7 ↦ ⟦ φ ↦ 𝜓.𝛼0 ⟧
        ν8 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν9 ⟧
        ν9 ↦ ⟦ Δ ↦ 0x002A ⟧
    ");
}

// // [x] > a        v1
// //   b > @        v2
// //     c          v3
// //       $.x
// // [x] > b        v4
// //   c > @        v5
// //     $.x
// // [x] > c        v6
// //   x > @
// // a              v7
// //   42           v8
// #[test]
// pub fn simulation_of_recursion() {
//     assert_emu!(7, 42, "
//         ν1 ↦ ⟦ φ ↦ ν2 ⟧
//         ν2 ↦ ⟦ φ ↦ ν4(𝜓), 𝛼0 ↦ ν3 ⟧
//         ν3 ↦ ⟦ φ ↦ ν6(𝜓), 𝛼0 ↦ ξ.𝛼0 ⟧
//         ν4 ↦ ⟦ φ ↦ ν5 ⟧
//         ν5 ↦ ⟦ φ ↦ ν6(𝜓), 𝛼0 ↦ ξ.𝛼0 ⟧
//         ν6 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
//         ν7 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν8 ⟧
//         ν8 ↦ ⟦ Δ ↦ 0x002A ⟧
//     ");
// }
//
// // [x] > a        v1
// //   b > @        v2
// //     f          v3
// //       $.x
// // [x] > b        v4
// //   c > @        v5
// //     f          v6
// //       $.x
// // [x] > c        v7
// //   f > @        v8
// //     $.x
// // [x] > f        v9
// //   x > @
// // a              v10
// //   42           v11
// #[test]
// pub fn deep_simulation_of_recursion() {
//     assert_emu!(10, 42, "
//         ν1 ↦ ⟦ φ ↦ ν2 ⟧
//         ν2 ↦ ⟦ φ ↦ ν4(𝜓), 𝛼0 ↦ ν3 ⟧
//         ν3 ↦ ⟦ φ ↦ ν9(𝜓), 𝛼0 ↦ ξ.𝛼0 ⟧
//         ν4 ↦ ⟦ φ ↦ ν5 ⟧
//         ν5 ↦ ⟦ φ ↦ ν7(𝜓), 𝛼0 ↦ ν6 ⟧
//         ν6 ↦ ⟦ φ ↦ ν9(𝜓), 𝛼0 ↦ ξ.𝛼0 ⟧
//         ν7 ↦ ⟦ φ ↦ ν8 ⟧
//         ν8 ↦ ⟦ φ ↦ ν9(𝜓), 𝛼0 ↦ ξ.𝛼0 ⟧
//         ν9 ↦ ⟦ φ ↦ ξ.𝛼0 ⟧
//         ν10 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν11 ⟧
//         ν11 ↦ ⟦ Δ ↦ 0x002A ⟧
//     ");
// }
//
// // [x] > foo        v1
// //   bool.if        v2
// //     int.less     v3
// //       $.x
// //       0          v4
// //     42           v5
// //     foo          v6
// //       int.sub    v7
// //         $.x
// //         1        v8
// // foo              v9
// //   7              v10
// #[test]
// pub fn simple_recursion() {
//     assert_emu!(9, 42, "
//         ν1 ↦ ⟦ φ ↦ ν2 ⟧
//         ν2 ↦ ⟦ λ ↦ bool.if, ρ ↦ ν3, 𝛼0 ↦ ν5, 𝛼1 ↦ ν6 ⟧
//         ν3 ↦ ⟦ λ ↦ int.less, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ν4 ⟧
//         ν4 ↦ ⟦ Δ ↦ 0x0000 ⟧
//         ν5 ↦ ⟦ Δ ↦ 0x002A ⟧
//         ν6 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν7 ⟧
//         ν7 ↦ ⟦ λ ↦ int.sub, ρ ↦ ξ.𝛼0, 𝛼0 ↦ ν8 ⟧
//         ν8 ↦ ⟦ Δ ↦ 0x0001 ⟧
//         ν9 ↦ ⟦ φ ↦ ν1(𝜓), 𝛼0 ↦ ν10 ⟧
//         ν10 ↦ ⟦ Δ ↦ 0x0007 ⟧
//     ");
// }
