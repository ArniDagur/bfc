# An optimising compiler for BF

[![Build Status](https://travis-ci.org/Wilfred/bfc.svg?branch=master)](https://travis-ci.org/Wilfred/bfc)

BFC is an optimising compiler for
[BF](https://en.wikipedia.org/wiki/Brainfuck).

It is written in Rust and uses LLVM.

```
BF source -> BFC IR -> LLVM IR -> x86_32 Binary
```

GPLv2 or later license.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc/generate-toc again -->
**Table of Contents**

- [An optimising compiler for BF](#an-optimising-compiler-for-bf)
    - [Compiling](#compiling)
    - [Usage](#usage)
    - [Running tests](#running-tests)
    - [Test programs](#test-programs)
    - [Optimisations](#optimisations)
        - [Combining Instructions](#combining-instructions)
        - [Loop Simplification](#loop-simplification)
    - [Other projects optimising BF](#other-projects-optimising-bf)

<!-- markdown-toc end -->


## Compiling

You will need LLVM and Rust beta installed to compile bfc.

    $ cargo build

## Usage

```
$ cargo run -- sample_programs/hello_world.bf
$ lli hello_world.ll
Hello World!
```

## Running tests

```
$ cargo test
```

## Test programs

There are a few test programs in this repo, but
http://www.hevanet.com/cristofd/brainfuck/tests.b is also an excellent
collection of test BF programs.

## Optimisations

bfc can use LLVM's optimisations, but it also offers some BF-specific
optimisations. There's a roadmap in
[optimisations.md](optimisations.md) of optimisations we will
implement at the BFC IR level.

### Combining Instructions

We combine successive increments/decrements:

```
   Compile             Combine
+++  =>   BFIncrement 1   =>   BFIncrement 3
          BFIncrement 1
          BFIncrement 1
```

If increments/decrements cancel out, we remove them entirely.

```
   Compile              Combine
+-   =>   BFIncrement  1    =>   # nothing!
          BFIncrement -1
```

We do the same thing for data increments/decrements:

```
   Compile                 Combine
>>>  =>   BFDataIncrement 1   =>   BFDataIncrement 3
          BFDataIncrement 1
          BFDataIncrement 1

   Compile                  Combine
><   =>   BFDataIncrement  1    =>   # nothing!
          BFDataIncrement -1
```

We do the same thing for successive sets:

```
       Combine
BFSet 1   =>   BFSet 2
BFSet 2

```

We combine sets and increments too:

```
  Compile            Known zero:         Combine
+   =>   BFIncrement 1   =>   BFSet 0      =>   BFSet 1
                              BFIncrement 1

```

We remove increments when there's a set immediately after:

```
             Combine
BFIncrement 1   =>   BFSet 2
BFSet 2

```

### Loop Simplification

`[-]` is a common BF idiom for zeroing cells. We replace that with
`BFSet`, enabling further instruction combination.

```
   Compile                Simplify
[-]  =>   BFLoop             =>   BFSet 0
            BFIncrement -1
```

### Dead Code Elimination

We remove loops that we know are dead.

For example, loops at the beginning of a program:

```
    Compile                    Known zero                 DCE
[>]   =>    BFLoop                 =>     BFSet 0          => BFSet 0
              BFDataIncrement 1           BFLoop
                                            BFDataIncrement 
```


Loops following another loop (one BF technique for comments is
`[-][this, is+a comment.]`).

```
      Compile                   Annotate                   DCE
[>][>]   =>  BFLoop                =>   BFLoop              =>   BFLoop
               BFDataIncrement 1          BFDataIncrement 1        BFDataIncrement 1
             BFLoop                     BFSet 0                  BFSet 0
               BFDataIncrement 1        BFLoop
                                          BFDataIncrement 1
```

We remove redundant set commands after loops (often generated by loop
annotation as above).

```
         Remove redundant set
BFLoop           =>   BFLoop
  BFIncrement -1        BFIncrement -1
BFSet 0

```

## Other projects optimising BF

There are also some interesting other projects for optimising BF
programs:

* https://code.google.com/p/esotope-bfc/wiki/Optimization
* http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html
* http://2π.com/10/brainfuck-using-llvm
