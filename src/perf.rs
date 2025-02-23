// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

use itertools::Itertools;
use std::collections::HashMap;
use std::fmt;

#[derive(Hash, Eq, PartialEq, strum_macros::Display)]
pub enum Transition {
    CPY,
    DEL,
    NEW,
    DLG,
    PPG,
    FND,
}

pub struct Perf {
    pub cycles: usize,
    pub peak: usize,
    pub atoms: HashMap<String, usize>,
    pub hits: HashMap<Transition, usize>,
    pub ticks: HashMap<Transition, usize>,
}

impl Default for Perf {
    fn default() -> Self {
        Self::new()
    }
}

impl Perf {
    pub fn new() -> Perf {
        Perf {
            atoms: HashMap::new(),
            ticks: HashMap::new(),
            hits: HashMap::new(),
            cycles: 0,
            peak: 0,
        }
    }

    pub fn tick(&mut self, t: Transition) {
        *self.ticks.entry(t).or_insert(0) += 1;
    }

    pub fn hit(&mut self, t: Transition) {
        *self.hits.entry(t).or_insert(0) += 1;
    }

    pub fn atom(&mut self, a: String) {
        *self.atoms.entry(a).or_insert(0) += 1;
    }

    pub fn peak(&mut self, s: usize) {
        if self.peak < s {
            self.peak = s
        }
    }

    pub fn total_hits(&self) -> usize {
        self.hits.values().sum()
    }

    pub fn total_ticks(&self) -> usize {
        self.ticks.values().sum()
    }

    pub fn total_atoms(&self) -> usize {
        self.atoms.values().sum()
    }
}

macro_rules! print {
    ($lines:expr, $title:expr, $list:expr, $total:expr) => {
        $lines.push(format!("{}:", $title));
        $lines.extend(
            $list
                .iter()
                .map(|(t, c)| format!("\t{}: {}", t, c))
                .sorted(),
        );
        $lines.push(format!("\tTotal: {}", $total));
    };
}

impl fmt::Display for Perf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];
        lines.push(format!("Cycles: {}", self.cycles));
        lines.push(format!("Peak: {}", self.peak));
        print!(lines, "Atoms", self.atoms, self.total_atoms());
        print!(lines, "Ticks", self.ticks, self.total_ticks());
        print!(lines, "Hits", self.hits, self.total_hits());
        f.write_str(lines.join("\n").as_str())
    }
}

#[test]
pub fn simple_increment() {
    let mut perf = Perf::new();
    perf.hit(Transition::DEL);
    assert!(perf.to_string().contains("DEL: 1"));
}

#[test]
pub fn sorts_them() {
    let mut perf = Perf::new();
    perf.hit(Transition::DEL);
    perf.hit(Transition::PPG);
    perf.hit(Transition::NEW);
    assert!(perf.to_string().contains("DEL: 1\n\tNEW: 1\n\tPPG: 1"));
}
