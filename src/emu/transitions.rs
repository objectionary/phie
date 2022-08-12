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
use crate::emu::{Emu, MAX_BASKETS, ROOT_BK, ROOT_OB};
use crate::loc::Loc;
use crate::locator::Locator;
use crate::object::{Ob, Object};
use crate::perf::{Perf, Transition};
use itertools::Itertools;
use log::trace;

macro_rules! join {
    ($log:expr) => {
        $log.iter().join("; ")
    };
}

impl Emu {
    /// Copy data from object to basket.
    pub fn copy(&mut self, perf: &mut Perf, bk: Bk) {
        let bsk = self.basket(bk);
        if let Some(Kid::Rqtd) = bsk.kids.get(&Loc::Phi) {
            let obj = self.object(bsk.ob);
            if let Some(d) = obj.delta {
                let _ = &self.baskets[bk as usize].put(Loc::Phi, Kid::Dtzd(d));
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
            let _ = &self.baskets[*b as usize].put(l.clone(), Kid::Dtzd(*d));
            perf.hit(Transition::PPG);
        }
        perf.tick(Transition::PPG);
    }

    /// Delete the basket if it's already finished.
    pub fn delete(&mut self, perf: &mut Perf, bk: Bk) {
        if bk == ROOT_BK {
            return;
        }
        let bsk = self.basket(bk);
        let obj = self.object(bsk.ob);
        if obj.constant {
            return;
        }
        let mut ready = true;
        for kid in bsk.kids.values() {
            if !matches!(kid, Kid::Empt) && !matches!(kid, Kid::Dtzd(_)) {
                ready = false;
                break;
            }
            if matches!(kid, Kid::Dtzd(_)) {
                for i in 0..self.baskets.len() {
                    let wbsk = self.basket(i as Bk);
                    if wbsk.is_empty() {
                        continue;
                    }
                    perf.tick(Transition::DEL);
                    for v in wbsk.kids.values() {
                        if let Kid::Wait(b, _) = v {
                            if *b == bk {
                                ready = false
                            }
                        }
                    }
                }
            }
        }
        if ready {
            self.baskets[bk as usize] = Basket::empty();
            trace!("delete(β{})", bk);
            perf.hit(Transition::DEL);
        }
        perf.tick(Transition::DEL);
    }

    /// Give control to the atom of the basket.
    pub fn delegate(&mut self, perf: &mut Perf, bk: Bk) {
        let bsk = self.basket(bk);
        if let Some(Kid::Rqtd) = bsk.kids.get(&Loc::Phi) {
            if !bsk.kids.values().any(|k| matches!(&k, Kid::Wait(_, _))) {
                let obj = self.object(bsk.ob);
                if let Some((n, func)) = &obj.lambda {
                    let name = n.clone();
                    perf.hit(Transition::DLG);
                    if let Some(d) = func(self, bk) {
                        perf.atom(name);
                        let _ = &self.baskets[bk as usize].put(Loc::Phi, Kid::Dtzd(d));
                        trace!("delegate(β{}) -> 0x{:04X}", bk, d);
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
            if let Some((locator, advice)) = obj.attrs.get(&loc) {
                let (tob, psi, attr) = self
                    .search(bk, locator)
                    .unwrap_or_else(|_| panic!("Can't find {} from β{}/ν{}", locator, bk, ob));
                let tpsi = if *advice { bk } else { psi };
                if let Some((pbk, ploc)) = attr {
                    let bsk = self.basket(pbk);
                    if let Some(Kid::Empt) = bsk.kids.get(&ploc) {
                        let _ = &self.baskets[pbk as usize]
                            .put(ploc.clone(), Kid::Wait(bk, loc.clone()));
                        let _ = &self.baskets[bk as usize].put(loc.clone(), Kid::Need(tob, tpsi));
                    } else {
                        let _ = &self.baskets[bk as usize]
                            .put(loc.clone(), Kid::Wait(pbk, ploc.clone()));
                    }
                } else {
                    let _ = &self.baskets[bk as usize].put(loc.clone(), Kid::Need(tob, tpsi));
                }
                perf.hit(Transition::FND);
            }
        }
        perf.tick(Transition::FND);
    }

    /// Make new basket for this attribute.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(&mut self, perf: &mut Perf, bk: Bk, loc: Loc) {
        if let Some(Kid::Need(tob, psi)) = self.basket(bk).kids.get(&loc) {
            let ob = self.basket(bk).ob;
            let nbk = if let Some(ebk) = self.stashed(*tob, *psi) {
                trace!("new(β{}/ν{}, {}) -> link to stashed β{}", bk, ob, loc, ebk);
                ebk
            } else {
                let id = self
                    .baskets
                    .iter()
                    .find_position(|b| b.is_empty())
                    .unwrap_or_else(|| {
                        panic!("No more empty baskets left in the pool of {}", MAX_BASKETS)
                    })
                    .0 as Bk;
                let mut bsk = Basket::start(*tob, *psi);
                for k in self.object(*tob).attrs.keys() {
                    bsk.put(k.clone(), Kid::Empt);
                }
                bsk.put(Loc::Phi, Kid::Rqtd);
                self.baskets[id as usize] = bsk;
                trace!("new(β{}/ν{}, {}) -> β{} created", bk, ob, loc, id);
                id
            };
            perf.hit(Transition::NEW);
            let _ = &self.baskets[bk as usize].put(loc.clone(), Kid::Wait(nbk, Loc::Phi));
        }
        perf.tick(Transition::NEW);
    }

    /// Suppose, the incoming locator is `^.0.@.2`. We have to find the right
    /// object in the catalog of them and return the position of the found one
    /// together with the suggested \psi.
    #[allow(clippy::type_complexity)]
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
                        return Err(format!("Object 𝜑 doesn't have ξ: {}", join!(log)));
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
                                "Can't find {} in ν{} and there is no 𝜑: {}",
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
            bsk.psi == psi
        }) {
            return Some(pos as Bk);
        }
        None
    }

    pub fn object(&self, ob: Ob) -> &Object {
        &self.objects[ob]
    }

    pub fn basket(&self, bk: Bk) -> &Basket {
        &self.baskets[bk as usize]
    }
}
