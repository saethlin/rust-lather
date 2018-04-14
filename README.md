# Installation Instructions

Currently, the only option is to build from source, which means you will need a Rust compiler. Head to rustup.rs and follow the instructions. By default, everything will be installed to your home directory. To install to some other directory, set the `CARGO_HOME` and `RUSTUP_HOME` environment variables to the desired installation path before running rustup.

Lather will build with the default rustup settings. Lather also requires a version of the C library `gsl`<sup>1</sup>, which is availble from apt as `libgsl0-dev`.

With the initial dependencies installed, run `python setup.py install` inside the source directory.

If you're building with a nightly Rust compiler, some nice SIMD optimizations are available behind a feature.

<sup>1</sup>Problems with C/C++ dependencies are the reason this is now written in Rust. I would like to remove this dependency as well, but I'm not quite up for re-implementing the very nice GSL interpolators.
