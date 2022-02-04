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
use simple_logger::SimpleLogger;
use std::str::FromStr;
use log::trace;
use crate::path::Item::Phi;

pub struct Emu {
    pub objects: [Object; 256],
    pub boxes: [Dabox; 256],
    pub total_boxes: usize,
}

impl Emu {
    /// Make an empty Emu, which you can later extend with
    /// additional objects.
    pub fn empty() -> Emu {
        Emu {
            objects: arr![Object::open(); 256],
            boxes: arr![Dabox::empty(); 256],
            total_boxes: 0,
        }
    }

    /// Add an additional object
    pub fn put(&mut self, pos: usize, obj: Object) -> &mut Emu {
        self.objects[pos] = obj;
        self
    }

    pub fn log(&self) {
        for i in self.objects.iter().enumerate() {
            let (ob, obj): (usize, &Object) = i;
            if obj.is_empty() {
                continue;
            }
            let bx = self.boxes.iter().position(|d| !d.is_empty() && d.object as usize == ob);
            trace!(
                "ν{} ⟦{}⟧{}",
                ob, obj,
                self.boxes.iter().enumerate()
                    .filter(|(i, d)| !d.is_empty() && d.object as usize == ob)
                    .map(|(i, d)| format!("\n\t➞ #{} ({})", i, d))
                    .collect::<Vec<String>>()
                    .join("")
            )
        }
    }

    /// Calculate sub-object of the object `ob` found by the
    /// path item `item`, using already created dataization box `bx`.
    pub fn calc(&mut self, bx: usize, item: Item) -> Data {
        let dbox = &self.boxes[bx];
        let ob = dbox.object as usize;
        let path = Path::from_item(item.clone());
        let target = match self.find(bx, &path) {
            Ok(p) => p,
            Err(e) => panic!("Can't find '{}' in ν{}: {}", path, ob, e),
        };
        let sub = self.new(target, dbox.xi);
        let data = self.dataize(sub);
        (&mut self.boxes[bx]).put_kid(item, data);
        self.delete(sub);
        data
    }

    /// Perform "dataization" procedure on a single box.
    pub fn dataize(&mut self, bx: usize) -> Data {
        let ob = self.boxes[bx].object as usize;
        let obj = &self.objects[ob];
        let xi = &(self.boxes[self.boxes[bx].xi].object as usize);
        if bx > 30 {
            panic!("Too many!");
        }
        trace!("\n\ndataize(#{} -> ν{}, 𝜉:#{})...", bx, ob, xi);
        self.log();
        let r = if obj.data.is_some() {
            obj.data.unwrap()
        } else if obj.kids.contains_key(&Item::Phi) {
            let phi = self.find(bx, &Path::from_item(Item::Phi)).unwrap();
            let sub = self.new(phi, self.boxes[bx].xi);
            let ret = self.dataize(sub);
            self.delete(sub);
            ret
        } else if obj.parent.is_some() {
            let b = self.new(obj.parent.unwrap(), self.boxes[bx].xi);
            (&mut self.boxes[bx]).put_xi(bx);
            let ret = self.dataize(b);
            self.delete(b);
            ret
        } else if obj.atom.is_some() {
            obj.atom.unwrap()(self, bx)
        } else {
            panic!("Can't dataize empty object #{}", bx)
        };
        (&mut self.boxes[bx]).put_ret(r);
        r
    }

    /// Make new dataization box and return its position ID.
    pub fn new(&mut self, ob: usize, xi: usize) -> usize {
        let dabox = Dabox::start(ob, xi);
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
    fn find(&self, bx: usize, path: &Path) -> Result<usize, String> {
        let dabox = &self.boxes[bx];
        let mut items = path.to_vec();
        let mut ret = Err("Nothing found".to_string());
        let mut last = 0;
        let mut obj : &Object = &self.objects[dabox.object as usize];
        loop {
            if items.is_empty() {
                break ret;
            }
            let item = items.remove(0);
            let next = match item {
                Item::Root => 0,
                Item::Xi => {
                    let dbox = &self.boxes[dabox.xi];
                    if dbox.is_empty() {
                        panic!("The 𝜉 of box #{} points to empty box #{}", bx, dabox.xi)
                    }
                    dbox.object as usize
                },
                Item::Obj(i) => i,
                Item::Attr(_) => match obj.kids.get(&item) {
                    None => match obj.parent {
                        None => return Err(format!("Can't find '{}' in ν{} and there is no ψ", item, last)),
                        Some(p) => self.find(
                            bx,
                            match self.objects[p].kids.get(&item) {
                                Some(p) => p,
                                None => return Err(format!("Can't get '{}' from ν{}, which is ψ of ν{}", item, p, last)),
                            }
                        )?
                    },
                    Some(p) => self.find(bx, p)?
                },
                _ => self.find(
                    bx,
                    match obj.kids.get(&item) {
                        Some(p) => p,
                        None => return Err(format!("Can't get '{}' from ν{}", item, last)),
                    }
                )?,
            };
            obj = &self.objects[next];
            last = next;
            ret = Ok(next)
        }
    }
}

#[test]
pub fn dataize_simple_data() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    let bx = emu.new(0, 0);
    assert_eq!(42, emu.dataize(bx));
}

#[test]
pub fn with_simple_decorator() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    emu.put(1, Object::open().with(Item::Phi, ph!("v0")));
    let bx = emu.new(1, 0);
    assert_eq!(42, emu.dataize(bx));
}

#[test]
pub fn finds_complex_path() {
    let mut emu = Emu::empty();
    emu.put(1, Object::open().with(Item::Phi, ph!("v2")));
    emu.put(2, Object::open().with(Item::Attr(3), ph!("v1")));
    emu.put(3, Object::open().with(Item::Attr(0), ph!("$.3.@")));
    let bx1 = emu.new(2, 0);
    let bx = emu.new(3, bx1);
    assert_eq!(2, emu.find(bx, &ph!("v3.0")).unwrap());
}

#[test]
pub fn finds_through_copy() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    emu.put(1, Object::open().with(Item::Attr(0), ph!("v0")));
    emu.put(3, Object::copy(1));
    let bx = emu.new(3, 0);
    assert_eq!(0, emu.find(bx, &ph!("$.0")).unwrap());
}

#[test]
pub fn finds_in_itself() {
    let mut emu = Emu::empty();
    emu.put(0, Object::dataic(42));
    emu.put(1, Object::open().with(Item::Phi, ph!("v0")));
    let bx = emu.new(1, 0);
    assert_eq!(0, emu.find(bx, &Path::from_item(Item::Phi)).unwrap());
}

#[test]
pub fn saves_ret_into_dabox() {
    let mut emu = Emu::empty();
    let d = 42;
    emu.put(0, Object::dataic(d));
    let bx = emu.new(0, 0);
    assert_eq!(d, emu.dataize(bx));
    assert!(emu.boxes[bx].to_string().contains(&String::from(format!("{:04X}", d))));
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
        Object::open()
            .with(Item::Phi, ph!("v1"))
            .with(Item::Attr(0), ph!("v0"))
            .with(Item::Attr(1), ph!("v0")),
    );
    emu.put(3, Object::open().with(Item::Phi, ph!("v2")));
    let bx = emu.new(3, 3);
    assert_eq!(84, emu.dataize(bx));
}
