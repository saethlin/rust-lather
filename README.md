# Installation Instructions

Currently, the only option is to build from source, which means you will need a Rust compiler. Head to rustup.rs and follow the instructions. By default, everything will be installed to your home directory. To install to some other directory, set the `CARGO_HOME` and `RUSTUP_HOME` environment variables to the desired installation path before running rustup. Rather will build with the defeault rustup settings. Rather also requires a version of the C library `gsl`<sup>1</sup>, and the Python package `setuptools_rust` which is readily availble through pip.

With the initial dependencies installed, run `python setup.py install` inside the source directory.

<sup>1</sup>Problems with C/C++ dependencies are the reason this is now written in Rust. I am working to remove this one because rust-gsl is not ABI-compatible with newer versions of the C library.
