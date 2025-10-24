// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

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
                debug!("dataize() -> 0x{:04X} in {:?}\n{}\n{}", *d, time.elapsed(), perf, self);
                return (*d, perf);
            }
            cycles += 1;
            if self.opts.contains(&Opt::StopWhenTooManyCycles) && cycles > MAX_CYCLES {
                panic!("Too many cycles ({}), most probably endless recursion:\n{}", cycles, self);
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
