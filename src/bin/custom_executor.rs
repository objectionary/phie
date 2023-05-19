// Copyright (c) 2023 Eugene Darashkevich
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

extern crate phie;

use phie::data::Data;
use phie::emu::{Emu, Opt};
use std::env;
use std::{
    fs,
    io::{self, BufRead, Error},
};

pub fn emulate(phi_code: &str) -> Data {
    let mut emu: Emu = phi_code.parse().unwrap();
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    emu.dataize().0
}

fn read_file_with_number(file_name: &str) -> Result<(i32, Vec<String>), Error> {
    let file = fs::File::open(file_name)?;
    let reader = io::BufReader::new(file);

    let lines: Vec<String> = reader.lines().map(|line| line.unwrap()).collect();

    let correct = match lines.first() {
        Some(line) => line.parse::<i32>().unwrap(),
        None => return Err(Error::new(io::ErrorKind::InvalidData, "Empty file")),
    };

    Ok((correct, lines[1..].to_vec()))
}

pub fn run_emulator(filename: &str) {
    match read_file_with_number(filename) {
        Ok((correct, lines)) => {
            let phi_code = lines.join("\n");
            let result = emulate(&phi_code);
            assert_eq!(i32::from(result), correct);
        }
        Err(error) => {
            println!("Error reading {}: {}", filename, error);
        }
    }
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let filename: &str = &args[1];
    run_emulator(filename);
}

#[test]
fn executes_file_example() {
    run_emulator("tests/resources/written_text_example");
}

#[test]
fn executes_fibonacci_file() {
    run_emulator("tests/resources/written_fibonacci_test");
}

#[test]
fn executes_sum_file() {
    run_emulator("tests/resources/written_sum_test");
}
