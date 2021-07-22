# Development

Please contribute [on github](https://github.com/kindly/flatterer).

## Local install

Need local rust stable toolchain and python-virtualenv.

Clone this repo.

Create python virtual environment.

```bash
virtualenv .ve
source .ve/bin/activate
```

Install maturin (the build tool for building rust extensions)

```bash
pip install maturin
```

Test local build.

```bash
maturin develop --release 
```

## Builds

Build anylinux wheels for all pythons 3.6+ and source distribution.

Run this once

```
sudo docker build -t flatterer-anylinux .
```

To build

```
sudo docker run  -v $(pwd):/io flatterer-anylinux build --release
```
