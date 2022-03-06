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

mod tests;
mod transitions;

use crate::basket::{Basket, Bk, Kid};
use crate::data::Data;
use crate::loc::Loc;
use crate::object::{Ob, Object};
use crate::perf::Perf;
use arr_macro::arr;
use log::{debug, trace};
use regex::Regex;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::time::Instant;

pub const ROOT_BK: Bk = 0;
pub const ROOT_OB: Ob = 0;

const MAX_CYCLES: usize = 65536;
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
            Some(Kid::Need(_, _)) | Some(Kid::Wait(_, _)) | Some(Kid::Rqtd) => None,
            Some(Kid::Dtzd(d)) => Some(*d),
        }
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
            perf.peak(self.baskets.iter().filter(|bsk| !bsk.is_empty()).count());
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
