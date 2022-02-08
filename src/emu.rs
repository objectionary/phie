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

use crate::atom::*;
use crate::dabox::{Bx, Dabox};
use crate::data::Data;
use crate::object::{Ob, Object};
use crate::path::{Item, Path};
use crate::ph;
use arr_macro::arr;
use std::str::FromStr;
use log::trace;
use itertools::Itertools;

pub const ROOT_BX: Bx = 0;
pub const ROOT_OB : Ob = 0;

pub struct Emu {
    pub objects: [Object; 256],
    pub boxes: [Dabox; 256],
    pub total_boxes: usize,
}

macro_rules! join {
    ($log:expr) => {
        $log.iter().join("; ")
    };
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional objects.
    pub fn empty() -> Emu {
        let mut emu = Emu {
            objects: arr![Object::open(); 256],
            boxes: arr![Dabox::empty(); 256],
            total_boxes: 0,
        };
        emu.put(ROOT_OB, Object::dataic(0));
        let bx = emu.new(ROOT_BX, ROOT_BX);
        assert_eq!(ROOT_BX, bx);
        emu
    }

    /// Add an additional object
    pub fn put(&mut self, ob: Ob, obj: Object) -> &mut Emu {
        assert!(self.objects[ob].is_empty(), "The object ν{} already occupied", ob);
        self.objects[ob] = obj;
        self
    }

    pub fn log(&self) {
        for i in self.objects.iter().enumerate() {
            let (ob, obj): (usize, &Object) = i;
            if obj.is_empty() {
                continue;
            }
            trace!(
                "ν{} ⟦{}⟧{}",
                ob, obj,
                self.boxes.iter().enumerate()
                    .filter(|(_, d)| !d.is_empty() && d.ob as usize == ob)
                    .map(|(i, d)| format!("\n\t➞ #{} ({})", i, d))
                    .collect::<Vec<String>>()
                    .join("")
            )
        }
    }

    /// Dataize object `attr` in the object represented by the
    /// dataization box `bx`.
    pub fn calc_attr(&mut self, bx: Bx, attr: Item) -> Result<Data, String> {
        let dbox = self.dabox(bx);
        let ob = dbox.ob;
        trace!("calc_attr(#{}, {}) -> ν{}...", bx, attr, ob);
        let obj = self.object(ob);
        let target = match self.find(bx, &Path::from_item(attr.clone())) {
            Ok(p) => p,
            Err(e) => panic!("Can't find in ν{}: {}", ob, e),
        };
        let psi = obj.attrs.get(&attr).unwrap().1;
        let xi = dbox.xi.clone();
        let sub = self.new(target, if psi { bx } else { xi });
        let ret = self.dataize(sub);
        (&mut self.boxes[bx]).put_kid(attr, ret.clone().unwrap());
        self.delete(sub);
        ret
    }

    /// Dataize object `attr` in the object represented by the
    /// dataization box `bx`. Doesn't delete the box, just
    /// dataize and save the result into its `ret` field.
    pub fn dataize(&mut self, bx: Bx) -> Result<Data, String> {
        let dbox = self.dabox(bx);
        let ob = dbox.ob;
        trace!("\n\ndataize(#{}) -> ν{}...", bx, ob);
        self.log();
        let obj = self.object(ob);
        let ret = if let Some(delta) = obj.delta {
            Ok(delta)
        } else if let Some(lambda) = obj.lambda {
            Ok(lambda(self, bx))
        } else if obj.attrs.contains_key(&Item::Phi) {
            self.calc_attr(bx, Item::Phi)
        } else {
            return Err(format!("Can't dataize empty object #{}", bx))
        };
        (&mut self.boxes[bx]).put_ret(ret.clone().unwrap());
        ret
    }

    /// Suppose, the incoming path is `^.0.@.2`. We have to find the right
    /// object in the catalog of them and return the position of the found one.
    fn find(&self, bx: Bx, path: &Path) -> Result<Ob, String> {
        let dbox = self.dabox(bx);
        let mut items = path.to_vec();
        let mut ret = Err("Nothing found".to_string());
        let mut last = 0;
        let mut obj: &Object = self.object(dbox.ob);
        let mut log = vec![];
        ret = loop {
            if items.is_empty() {
                break ret;
            }
            let item = items.remove(0);
            log.push(item.to_string());
            let next = match item {
                Item::Root => ROOT_BX,
                Item::Xi => {
                    let ob = self.dabox(dbox.xi).ob;
                    log.push(format!("ξ=ν{}", ob));
                    ob
                },
                Item::Obj(i) => i,
                Item::Attr(_) => match obj.attrs.get(&item) {
                    None => match obj.attrs.get(&Item::Phi) {
                        None => return Err(format!("Can't find '{}' in ν{} and there is no φ: {}", item, last, join!(log))),
                        Some((p, _psi)) => {
                            items.insert(0, item);
                            items.splice(0..0, p.to_vec());
                            log.push(format!("++{}", p));
                            last
                        }
                    },
                    Some((p, _psi)) => {
                        items.splice(0..0, p.to_vec());
                        log.push(format!("+{}", p));
                        last
                    }
                },
                _ => self.find(
                    bx,
                    match obj.attrs.get(&item) {
                        Some((p, _psi)) => p,
                        None => return Err(format!("Can't get '{}' from ν{}: {}", item, last, join!(log))),
                    }
                )?,
            };
            obj = self.object(next);
            last = next;
            ret = Ok(next)
        };
        trace!("find(#{}/ν{}, {}) -> ν{}\n\t{}", bx, dbox.ob, path, ret.clone().unwrap(), join!(log));
        ret
    }

    /// Make new dataization box and return its position ID.
    pub fn new(&mut self, ob: Ob, xi: Bx) -> Bx {
        let dbox = Dabox::start(ob, xi);
        let pos = self.total_boxes;
        if self.total_boxes > 30 {
            panic!("Too many")
        }
        self.total_boxes += 1;
        self.boxes[pos] = dbox;
        pos
    }

    /// Delete dataization box.
    pub fn delete(&mut self, bx: Bx) {
        self.boxes[bx] = Dabox::empty();
    }

    fn object(&self, ob: Ob) -> &Object {
        &self.objects[ob]
    }

    fn dabox(&self, bx: Bx) -> &Dabox {
        &self.boxes[bx]
    }
}

#[test]
pub fn dataize_simple_data() {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(42));
    let bx = emu.new(1, ROOT_BX);
    assert_eq!(42, emu.dataize(bx).unwrap());
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Item::Phi, ph!("v1"), false));
    let bx = emu.new(2, ROOT_BX);
    assert_eq!(42, emu.dataize(bx).unwrap());
}

#[test]
pub fn with_many_decorators() {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Item::Phi, ph!("v1"), false));
    emu.put(3, Object::open().with(Item::Phi, ph!("v2"), false));
    emu.put(4, Object::open().with(Item::Phi, ph!("v3"), false));
    let bx = emu.new(4, ROOT_BX);
    assert_eq!(42, emu.dataize(bx).unwrap());
}

// v1 -> [ @ -> v2 ]
// v2 -> [ a3 -> v1 ]
// v3 -> [ a0 -> $.a3.@ ]
#[test]
pub fn finds_complex_path() {
    let mut emu = Emu::empty();
    emu.put(1, Object::open().with(Item::Phi, ph!("v2"), false));
    emu.put(2, Object::open().with(Item::Attr(3), ph!("v1"), false));
    emu.put(3, Object::open().with(Item::Attr(0), ph!("$.3.@"), false));
    let bx2 = emu.new(2, ROOT_BX);
    let bx3 = emu.new(3, bx2);
    assert_eq!(2, emu.find(bx3, &ph!("v3.0")).unwrap());
}

// v1 -> [ D -> 42 ]
// v2 -> [ a0 -> v1 ]
// v3 -> [ @ -> v2 ]
#[test]
pub fn finds_through_copy() {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Item::Attr(0), ph!("v1"), false));
    emu.put(3, Object::open().with(Item::Phi, ph!("v2"), true));
    let bx2 = emu.new(3, ROOT_BX);
    let bx3 = emu.new(3, bx2);
    assert_eq!(1, emu.find(bx3, &ph!("$.0")).unwrap());
}

#[test]
pub fn finds_in_itself() {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(42));
    emu.put(2, Object::open().with(Item::Phi, ph!("v1"), false));
    let bx = emu.new(2, ROOT_BX);
    assert_eq!(1, emu.find(bx, &Path::from_item(Item::Phi)).unwrap());
}

#[test]
pub fn saves_ret_into_dabox() {
    let mut emu = Emu::empty();
    let d = 42;
    emu.put(1, Object::dataic(d));
    let bx = emu.new(1, ROOT_BX);
    assert_eq!(d, emu.dataize(bx).unwrap());
    assert!(emu.boxes[bx].to_string().contains(&String::from(format!("{:04X}", d))));
}

// v1 -> [ D -> 42 ]
// v2 -> [ λ -> int_add, ^ -> $.0, a0 -> $.a1 ]
// v3 -> [ @ -> v2(𝜓), a0 -> v1, a1 -> v1 ]
// v4 -> [ @ -> v3(𝜓) ]
#[test]
pub fn summarizes_two_numbers() {
    let mut emu = Emu::empty();
    emu.put(1, Object::dataic(42));
    emu.put(
        2,
        Object::atomic(int_add)
            .with(Item::Rho, ph!("$.0"), false)
            .with(Item::Attr(0), ph!("$.1"), false),
    );
    emu.put(
        3,
        Object::open()
            .with(Item::Phi, ph!("v2"), true)
            .with(Item::Attr(0), ph!("v1"), false)
            .with(Item::Attr(1), ph!("v1"), false),
    );
    emu.put(4, Object::open().with(Item::Phi, ph!("v3"), true));
    let bx = emu.new(3, ROOT_BX);
    assert_eq!(84, emu.dataize(bx).unwrap());
}

// [x] > a
//   $.x > @
// a > foo
//   a 42 > @
//
// v1 -> [ @ -> $.a0 ]
// v2 -> [ D -> 42 ]
// v3 -> [ @ -> v1(𝜓), a0 -> v2 ]
// v4 -> [ @ -> v1(𝜓), a0 -> v3 ]
#[test]
pub fn calls_itself_once() {
    let mut emu = Emu::empty();
    emu.put(
        1,
        Object::open()
            .with(Item::Phi, ph!("$.0"), false)
    );
    emu.put(2, Object::dataic(42));
    emu.put(
        3,
        Object::open()
            .with(Item::Phi, ph!("v1"), true)
            .with(Item::Attr(0), ph!("v2"), false)
    );
    emu.put(
        4,
        Object::open()
            .with(Item::Phi, ph!("v1"), true)
            .with(Item::Attr(0), ph!("v3"), false)
    );
    let bx = emu.new(3, ROOT_BX);
    assert_eq!(42, emu.dataize(bx).unwrap());
}

// [x] > a
//   $.x > @
// [y] > b
//   a > @
//     $.y
// b 42 > foo
//
// v1 -> [ @ -> $.a0 ]
// v2 -> [ @ -> v3(𝜓) ]
// v3 -> [ @ -> v1(𝜓), a0 -> $.a0 ]
// v4 -> [ D -> 42 ]
// v5 -> [ @ -> v2(𝜓), a0 -> v4 ]
#[test]
pub fn injects_xi_correctly() {
    let mut emu = Emu::empty();
    emu.put(
        1,
        Object::open()
            .with(Item::Phi, ph!("$.0"), false)
    );
    emu.put(
        2,
        Object::open()
            .with(Item::Phi, ph!("v3"), true)
    );
    emu.put(
        3,
        Object::open()
            .with(Item::Phi, ph!("v1"), true)
            .with(Item::Attr(0), ph!("$.0"), false)
    );
    emu.put(4, Object::dataic(42));
    emu.put(
        5,
        Object::open()
            .with(Item::Phi, ph!("v2"), true)
            .with(Item::Attr(0), ph!("v4"), false)
    );
    let bx = emu.new(5, ROOT_BX);
    assert_eq!(42, emu.dataize(bx).unwrap());
}
