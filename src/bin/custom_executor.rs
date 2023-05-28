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
use std::fs;
use std::str::FromStr;

fn emulate(phi_code: &str) -> Data {
    let mut emu: Emu = Emu::from_str(phi_code).unwrap();
    emu.opt(Opt::LogSnapshots);
    emu.opt(Opt::StopWhenTooManyCycles);
    emu.opt(Opt::StopWhenStuck);
    emu.dataize().0
}

pub fn run_emulator(filename: &str) -> i16 {
    let binding = fs::read_to_string(filename).unwrap();
    let phi_code: &str = binding.as_str();
    emulate(&phi_code)
}

pub fn execute_program(args: &[String]) -> i16 {
    assert!(args.len() >= 2);
    let filename: &str = &args[1];
    let result: i16 = run_emulator(filename);
    if args.len() >= 3 {
        let correct = args[2].parse::<i16>().unwrap();
        assert_eq!(result, correct);
    }
    result
}

pub fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2);
    let result = execute_program(&args);
    println!("Executor result: {}", result);
}

#[test]
#[should_panic]
fn test_main() {
    main();
}

#[test]
fn test_execute_program_with_valid_args() {
    let args = vec![
        "program_name".to_string(),
        "tests/resources/written_test_example".to_string(),
        "84".to_string(),
    ];
    let result = execute_program(&args);
    assert_eq!(result, 84);
}

#[test]
#[should_panic]
fn test_execute_program_with_invalid_args() {
    let args = vec!["program_name".to_string()];
    execute_program(&args);
}

#[test]
fn executes_file_example() {
    assert_eq!(84, run_emulator("tests/resources/written_test_example"));
}

#[test]
fn executes_fibonacci_file() {
    assert_eq!(21, run_emulator("tests/resources/written_fibonacci_test"));
}

#[test]
fn executes_sum_file() {
    assert_eq!(84, run_emulator("tests/resources/written_sum_test"));
}
