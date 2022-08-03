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

use crate::basket::{Bk, Kid};
use crate::data::Data;
use crate::emu::{Emu, Opt, ROOT_BK};
use crate::loc::Loc;
use crate::perf::Perf;
use log::debug;
use std::time::Instant;

const MAX_CYCLES: usize = 65536;

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
        for k in self.basket(bk).kids.keys() {
            keys.push(k.clone());
        }
        keys
    }
}
