#!/bin/bash
set -e -x

cargo build --release

# Compile wheels
/opt/python/cp35-cp35m/bin/pip wheel /io/ -w /io/wheelhouse

# Bundle external shared libraries into the wheels
for whl in wheelhouse/lather*.whl; do
    auditwheel repair "$whl" --plat manylinux2010_x86_64 -w /io/wheelhouse/
done
