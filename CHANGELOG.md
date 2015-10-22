# v1.3.0 (unreleased)

Performance:

* We now specify the data layout and target to LLVM, as recommended
  by the LLVM team. In principle this is faster, but we've seen no
  measurable performance boost.

Compatibility:

* bfc now provides up to 100,000 cells. This has been increased to
  support awib, which requires at least 65,535 cells available.

Bug fixes:

* Fixed a compiler crash due to bounds analysis ignoring offsets.

# v1.2.0

Optimisations:

* Compile time execution is now much smarter. Previously, we had to
finish executing loops in order to skip runtime execution. We can now
partly execute loops at runtime. This is a big help to many programs
with a large outer loop, previously they did not benefit from compile
time exeuction.

Compiler performance:

* `--dump-bf` is now much faster.

Bug fixes:

* In some cases, reorder with offset led to miscompilation
(only seen in mandelbrot.bf).

# v1.1.0

Optimisations:

* New optimisation: reorder with offset. See the readme for more
  details.
* Remove redundant sets, dead loop removal and combine before read are
  now smarter. Previously they required adjacent instructions, but
  they now find the next relevant instruction when there are
  irrelevant intermediate instructions.
* LLVM optimisation level can now be set with `--llvm-opt`.

Bug fixes:

* Fixed an issue with writing to stdout during speculative execution
  (we were writing to stdin instead).

Usability:

* Improved the output of `--help`
* `--dump-bf-ir` has been renamed to `--dump-bf`

# v1.0.0

First release!

* Compiles to 32-bit x86 binaries.
* Peephole optimisations
* Cell bounds analysis
* Speculative execution