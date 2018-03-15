import pip
pip.main(['install', 'setuptools-rust'])

from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name='lather',
    version='0.0.0',
    description='A Python extension for modeling starspot effects on radial'
    'velocity and photometric observations, inspired by complaints about the'
    'SOAP project of similar goals.',
    url='https://github.com/saethlin/rust-lather',
    author='Benjamin Kimock',
    author_email='kimockb@gmail.com',
    license='MIT/Apache-2.0',
    keywords='science simulation astronomy',
    classifiers=[
        'Development Status :: 3 - Alpha',
    ],
    rust_extensions=[
        RustExtension(
            'lather.lather', 'Cargo.toml', binding=Binding.RustCPython)
    ],
    packages=['lather'],
    zip_safe=False,
    install_requires=['numpy'],
)
