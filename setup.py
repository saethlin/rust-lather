from setuptools import setup

def build_native(spec):
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='./rust'
    )

    spec.add_cffi_module(
        module_path='lather._native',
        dylib=lambda: build.find_dylib('lather', in_path='target/release'),
        header_filename=lambda: build.find_header('lather.h', in_path='target'),
        rtld_flags=['NOW', 'NODELETE']
    )


setup(
    name='lather',
    version='0.0.1',
    packages=['lather'],
    zip_safe=False,
    platforms='any',
    setup_requires=['milksnake'],
    install_requires=['milksnake', 'numpy', 'scipy', 'opencv-python'],
    milksnake_tasks=[
        build_native
    ],
    scripts=['lather/lather-vis'],
)
