// SPDX-FileCopyrightText: Copyright (c) 2022 Yegor Bugayenko
// SPDX-License-Identifier: MIT

mod dataization;
mod tests;
mod tests_transitions;
mod transitions;

use crate::basket::{Basket, Bk, Kid};
use crate::data::Data;
use crate::loc::Loc;
use crate::object::{Ob, Object};
use arr_macro::arr;
use log::trace;
use regex::Regex;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

pub const ROOT_BK: Bk = 0;
pub const ROOT_OB: Ob = 0;

const MAX_OBJECTS: usize = 16;
const MAX_BASKETS: usize = 128;

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
                    .filter(|(_, d)| !d.is_empty() && d.ob == ob)
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
        let re_line = Regex::new("ν(\\d+)\\(𝜋\\) ↦ (⟦.*⟧)").unwrap();
        for line in s.trim().split('\n').map(|t| t.trim()) {
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
            objects: arr![Object::open(); 16],
            baskets: arr![Basket::empty(); 128],
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

    /// Inject a basket
    pub fn inject(&mut self, bk: Bk, bsk: Basket) -> &mut Emu {
        assert!(
            self.baskets[bk as usize].is_empty(),
            "The basket β{} already occupied",
            bk
        );
        self.baskets[bk as usize] = bsk;
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
