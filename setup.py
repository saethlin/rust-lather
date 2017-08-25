from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(name='lather',
    version='0.1.0',
    rust_extensions=[
        RustExtension('lather.lather', 'Cargo.toml', binding=Binding.RustCPython)
    ],
    packages=['lather'],
    zip_safe=False,
    install_requires=['numpy']
)
