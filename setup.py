from setuptools import setup


setup(
    name='pyhackernews',
    version='0.1',
    description='mimicking HN look and feel in terminal',
    long_description=open('README.rst').read(),
    license=open('LICENSE').read(),
    keywords='hackernews hn terminal',
    author='Dan Claudiu Pop',
    author_email='dancladiupop@gmail.com',
    url='https://github.com/danclaudiupop/hackernews/',
    packages=['hn'],
    install_requires=['urwid', 'beautifulsoup4'],
    classifiers=[
        'Environment :: Console',
        'License :: OSI Approved :: MIT License',
        'Natural Language :: English',
        'Operating System :: Unix',
        'Programming Language :: Python :: 2.7',
        'Topic :: Terminals',
    ],
    entry_points={
        'console_scripts': ['hn = hn.core:live']
    })
