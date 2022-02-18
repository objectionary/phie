[![make](https://github.com/yegor256/eoc/actions/workflows/cargo.yml/badge.svg)](https://github.com/yegor256/eoc/actions/workflows/cargo.yml)

It's an experimental compiler of [EO](https://www.eolang.org) to binaries.

To build it, install [Rust](https://www.rust-lang.org/tools/install) and then:

```bash
$ cargo build --release
```

If everything goes well, an executable binary will be in `target/release/fibonacci`:

```bash
$ target/release/fibonacci 7 40
```

This will calculate the 7th Fibonacci number 40 times.
Don't try to play with much larger numbers, this binary code is very slow. It's just an experiment.

To compiler your own program instead of Fibonacci calculator, you have to 
convert EO code into Rust structures, and then... nah, you can't do it, sorry.
