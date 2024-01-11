# SysY-Compiler

## Introduction

This project is a compiler for SysY language, which is a subset of C. It is implemented in `Rust` and uses `Koopa` as intermediate representation.

The compiler can compile SysY source code into Koopa IR with `-koopa` option and RISC-V assembly with `-riscv` option.

Even though the compiler can satisfy the standard of course, there are still some bugs and flaws. I am glad to receive any suggestions and corrections.

## Architecture

Building a compiler is a complex task. With the help of automatic tools, I devote most of my effort in parts below.

- `mem`: create Koopa IR in memory, based on the AST. In this process, the compiler collects information about the birth and death of values, and large arrays initialized with zero.
- `ir`: translate Koopa IR in memory into string.
- `asm`: generate RISC-V assembly from Koopa IR in memory. Register allocation and other optimization is done in this process.
