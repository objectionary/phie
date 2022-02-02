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
use crate::dabox::Dabox;
use crate::data::Data;
use crate::object::Object;
use crate::path::{Item, Path};
use crate::ph;
use arr_macro::arr;
use std::str::FromStr;

pub struct Emu {
    pub objects: [Object; 256],
    pub boxes: [Dabox; 256],
    pub total_boxes: usize,
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional OBSes.
    pub fn empty() -> Emu {
        Emu {
            objects: arr![Object::abstrct(); 256],
            boxes: arr![Dabox::empty(); 256],
            total_boxes: 0,
        }
    }

    /// Add an additional object
    pub fn put(&mut self, pos: usize, obj: Object) -> &mut Emu {
        self.objects[pos] = obj;
        self
    }

    /// Calculate sub-object of the object `ob` found by the
    /// path item `item`, using already created dataization box `bx`.
    pub fn calc(&mut self, ob: usize, item: Item, bx: usize) -> Data {
        let dabox = &self.boxes[bx];
        let obj = &self.objects[ob];
        let path = match obj.kids.get(&item) {
            Some(p) => p,
            None => panic!("Can't find kid #{} in object #{}", item, ob),
        };
        let target = match self.find(bx, path) {
            Some(t) => t,
            None => panic!(
                "Can't find '{}' from the object #{} (𝜉=#{})",
                path, ob, dabox.xi
            ),
        };
        let sub = self.new(target, dabox.xi);
        let data = self.dataize(sub);
        self.delete(sub);
        data
    }

    /// Perform "dataization" procedure on a single box.
    pub fn dataize(&mut self, bx: usize) -> Data {
        let ob = self.boxes[bx].object;
        let obj = &self.objects[ob];
        if obj.open {
            (&mut self.boxes[bx]).xi = bx;
        }
        let r = if obj.data.is_some() {
            obj.data.unwrap()
        } else if obj.kids.contains_key(&Item::Phi) {
            self.calc(ob, Item::Phi, bx)
        } else if obj.atom.is_some() {
            obj.atom.unwrap()(self, ob, bx)
        } else {
            panic!("Can't dataize empty object #{}", bx)
        };
        (&mut self.boxes[bx]).ret = r;
        r
    }

    /// Make new dataization box and return its position ID.
    pub fn new(&mut self, obj: usize, xi: usize) -> usize {
        let dabox = Dabox::start(obj, xi);
        let pos = self.total_boxes;
        self.total_boxes += 1;
        self.boxes[pos] = dabox;
        pos
    }

    /// Delete dataization box.
    pub fn delete(&mut self, bx: usize) {
        self.boxes[bx] = Dabox::empty();
    }

    /// Suppose, the incoming path is `^.0.@.2`. We have to find the right
    /// object in the catalog of them and return the position of the found one.
    pub fn find(&self, bx: usize, path: &Path) -> Option<usize> {
        let dabox = &self.boxes[bx];
        let mut items = path.to_vec();
        let mut ret = None;
        let mut obj: &Object = &self.objects[0];
        loop {
            if items.is_empty() {
                break ret;
            }
            let item = items.remove(0);
            let next = match item {
                Item::Root => 0,
                Item::Xi => dabox.xi,
                Item::Obj(i) => i,
                _ => self.find(bx, obj.kids.get(&item)?)?,
            };
            obj = self.objects.get(next)?;
            ret = Some(next)
        }
    }
}

#[test]
pub fn dataize_simple_data() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    assert_eq!(42, emu.dataize(0));
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    let mut kid = Object::abstrct();
    kid.push(Item::Phi, "v1".parse().unwrap());
    emu.put(1, kid);
    assert_eq!(42, emu.dataize(1));
}

#[test]
pub fn finds_complex_path() {
    let mut emu = Emu::empty();
    emu.put(1, Object::abstrct().with(Item::Phi, ph!("v2")));
    emu.put(2, Object::abstrct().with(Item::Attr(3), ph!("v1")));
    emu.put(3, Object::abstrct().with(Item::Attr(0), ph!("$.3.@")));
    let bx = emu.new(3, 2);
    assert_eq!(2, emu.find(bx, &ph!("v3.0")).unwrap());
}

#[test]
pub fn saves_ret_into_dabox() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    let bx = emu.new(0, 0);
    assert_eq!(42, emu.dataize(bx));
    assert_eq!(42, emu.boxes[bx].ret);
}

#[test]
pub fn summarizes_two_numbers() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    emu.put(
        1,
        Object::atomic(int_add)
            .with(Item::Rho, ph!("$.0"))
            .with(Item::Attr(0), ph!("$.1")),
    );
    emu.put(
        2,
        Object::abstrct()
            .with(Item::Phi, ph!("v1"))
            .with(Item::Attr(0), ph!("v0"))
            .with(Item::Attr(1), ph!("v0")),
    );
    emu.put(3, Object::abstrct().with(Item::Phi, ph!("v2")));
    let bx = emu.new(3, 3);
    assert_eq!(84, emu.dataize(bx));
}
