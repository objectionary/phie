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

use lazy_static::lazy_static;
use regex::Regex;
use rstest::rstest;
use std::fmt;
use std::i8;
use std::str::{self, FromStr};

#[derive(Debug, Clone)]
pub struct RegisterError {
    msg: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Register(usize);

impl Register {
    pub fn num(&self) -> usize {
        self.0
    }
}

impl str::FromStr for Register {
    type Err = RegisterError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^#[0-9A-F]$").unwrap();
        }
        if !RE.is_match(s) {
            return Err(RegisterError {
                msg: format!("Invalid register '{}'", s),
            });
        }
        Ok(Register(usize::from_str_radix(&s[1..], 16).unwrap()))
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&*format!("#{:X}", self.0))
    }
}

#[rstest(
    txt,
    case("#0"),
    case("#9"),
    case("#F"),
    #[should_panic] case("#"),
    #[should_panic] case("#15"),
    #[should_panic] case("#f"),
    #[should_panic] case("# 99"),
    #[should_panic] case("bad syntax")
)]
fn parses_all_texts(txt: String) {
    assert_eq!(Register::from_str(&txt).unwrap().to_string(), txt)
}

#[test]
fn returns_reg_index() {
    assert_eq!(Register::from_str("#F").unwrap().num(), 15)
}
