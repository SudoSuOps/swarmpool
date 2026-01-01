"""
Merlin — SwarmOS Controller Daemon
"""

from setuptools import setup, find_packages

setup(
    name="merlin-swarmos",
    version="0.1.0",
    description="SwarmOS Controller Daemon",
    author="SudoHash LLC",
    author_email="dev@sudohash.com",
    url="https://github.com/sudohash/merlin",
    packages=find_packages(),
    python_requires=">=3.10",
    install_requires=[
        "aiohttp>=3.9.0",
        "eth-account>=0.10.0",
        "eth-utils>=2.3.0",
        "loguru>=0.7.0",
    ],
    entry_points={
        "console_scripts": [
            "merlin=merlin.cli:cli",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
)
