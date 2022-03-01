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

use std::collections::HashMap;
use std::fmt;

#[derive(Hash, Eq, PartialEq, strum_macros::Display)]
pub enum Transition {
    CPY,
    DEL,
    NEW,
    DLG,
    PPG,
}

pub struct Perf {
    pub cycles: usize,
    pub hits: HashMap<Transition, usize>,
    pub ticks: HashMap<Transition, usize>,
}

impl Perf {
    pub fn new() -> Perf {
        Perf {
            ticks: HashMap::new(),
            hits: HashMap::new(),
            cycles: 0,
        }
    }

    pub fn tick(&mut self, t: Transition) {
        *self.ticks.entry(t).or_insert(0) += 1;
    }

    pub fn hit(&mut self, t: Transition) {
        *self.hits.entry(t).or_insert(0) += 1;
    }

    pub fn total_hits(&self) -> usize {
        self.hits.values().fold(0, |sum, x| sum + x)
    }

    pub fn total_ticks(&self) -> usize {
        self.ticks.values().fold(0, |sum, x| sum + x)
    }
}

impl fmt::Display for Perf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec![];
        lines.push(format!("Cycles: {}", self.cycles));
        lines.push("Ticks:".to_string());
        lines.extend(self.ticks.iter().map(|(t, c)| format!("\t{}: {}", t, c)));
        lines.push(format!("\tTotal: {}", self.total_ticks()));
        lines.push("Hits:".to_string());
        lines.extend(self.hits.iter().map(|(t, c)| format!("\t{}: {}", t, c)));
        lines.push(format!("\tTotal: {}", self.total_hits()));
        f.write_str(lines.join("\n").as_str())
    }
}

#[test]
pub fn simple_increment() {
    let mut perf = Perf::new();
    perf.hit(Transition::DEL);
    assert!(perf.to_string().contains("DEL: 1"));
}
