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

use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct XMIR {
    #[serde(rename = "$value")]
    pub objects: Vec<Oabs>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Obj {
    Oabs(Oabs),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Oabs {
    #[serde(rename = "$abstract")]
    pub abs: Option<String>,
    pub line: u32,
    pub pos: u32,
    pub name: String,
    pub base: Option<String>,
    #[serde(rename = "o")]
    pub os: Vec<O>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct O {
    pub line: u32,
    pub pos: u32,
    pub name: String,
    pub base: Option<String>,
}

pub fn xmir_from_file(path: &str) -> XMIR {
    let mut sxmir: String = "".to_string();
    File::open(path)
        .unwrap()
        .read_to_string(&mut sxmir)
        .unwrap();
    println!("STR_XMIR: {}\n", sxmir);
    let xmir: XMIR = from_str(sxmir.as_str()).unwrap();
    println!("{:#?}\n", xmir);
    xmir
}
