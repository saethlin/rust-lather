from setuptools import setup
from setuptools_rust import Binding, RustExtension


setup(name='rather',
    version='0.1.0',
    rust_extensions=[
        RustExtension('rather._rather', 'Cargo.toml', binding=Binding.RustCPython)
    ],
    packages=['rather'],
    zip_safe=False)  # rust extensions are not zip safe, just like C-extensions.
