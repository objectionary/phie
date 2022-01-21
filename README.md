[![new](https://github.com/yegor256/phiPUemu/actions/workflows/make.yml/badge.svg)](https://github.com/yegor256/phiPUemu/actions/workflows/make.yml)

It'a s simple virtual machine that can execute 
[EO](https://www.eolang.org) programs. 

To build it, just make sure your code is in `program.h` and then run:

```bash
$ make
```

If everything goes well, an executable binary will be in `program.a`.

To analyze the quality of the code in this repo and make sure
there are no hidden defects, run all static analyzers and style checkers:

```bash
$ make sa
```

You need to have [Clang](https://clang.llvm.org),
[Make](https://www.gnu.org/software/make/),
[Clang-Tidy](https://clang.llvm.org/extra/clang-tidy/),
and [cpplint](https://github.com/cpplint/cpplint) installed.
