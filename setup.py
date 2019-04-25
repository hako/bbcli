from setuptools import setup


setup(
    name='bbcli',
    version='0.2.8',
    description='Browse BBC News like a hacker. (based on pyhackernews)',
    long_description=open('README.md').read(),
    long_description_content_type="text/markdown",
    license="MIT",
    keywords='bbc news console terminal curses urwid',
    author='Wesley Hill, Calvin Hill',
    author_email='wesley@hakobaito.co.uk',
    url='https://github.com/hako/bbcli',
    packages=['bbcli'],
    install_requires=['urwid', 'requests', 'arrow'],
    classifiers=[
        'Environment :: Console',
        'License :: OSI Approved :: MIT License',
        'Natural Language :: English',
        'Operating System :: Unix',
        'Programming Language :: Python :: 2',
        'Programming Language :: Python :: 3',
        'Topic :: Terminals',
    ],
    use_2to3=True,
    entry_points={
        'console_scripts': ['bbcli = bbcli.core:live']
    })
