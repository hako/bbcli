from setuptools import setup


setup(
    name='bbcli',
    version='0.2.10',
    description='Browse BBC News like a hacker. (based on pyhackernews)',
    long_description=open('README.md').read(),
    long_description_content_type="text/markdown",
    license="MIT",
    keywords='bbc news console terminal curses urwid',
    author='Wesley Hill, Calvin Hill',
    author_email='wesley@hakobaito.co.uk',
    url='https://github.com/hako/bbcli',
    packages=['bbcli'],
    install_requires=['urwid==2.1.1', 'requests==2.31.0', 'arrow==0.15.8'],
    classifiers=[
        'Environment :: Console',
        'License :: OSI Approved :: MIT License',
        'Natural Language :: English',
        'Operating System :: Unix',
        'Programming Language :: Python :: 2',
        'Programming Language :: Python :: 3',
        'Topic :: Terminals',
    ],
    entry_points={
        'console_scripts': ['bbcli = bbcli.core:live']
    })
