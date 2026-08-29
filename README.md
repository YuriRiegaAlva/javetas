# javetas

A tiny CLI that scaffolds simple Java projects for learning. No Maven, no Gradle, no magic — just `javac`, `java`, and a Makefile you can read.

## Requirements

- A JDK (Java 25 or newer) to compile and run the generated projects.

## Install

**Linux / macOS**

```sh
curl -fsSL https://raw.githubusercontent.com/YuriRiegaAlva/javetas/main/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/YuriRiegaAlva/javetas/main/install.ps1 | iex
```

The binary is downloaded from the [latest release](https://github.com/YuriRiegaAlva/javetas/releases) and installed to `~/.local/bin` (Linux/macOS) or `%LOCALAPPDATA%\javetas` (Windows). No Rust toolchain needed.

## Usage

```sh
javetas new demo                        # create a project (interactive if needed)
javetas new demo --package com.ejemplo  # with a package

cd demo
javetas add Persona                     # add a class to the project
javetas add Persona --package com.otro  # add a class in its own package
javetas add Contrato --interface        # add an interface instead of a class
javetas build                           # compile into out/
javetas run                             # compile and run Main
javetas run Persona                     # run a different class
javetas run com.otro.Persona            # run a class in another package
javetas update                          # self-update to the latest release
javetas update --yes                    # same, without asking
```

Each generated project contains a `README.md` that explains how `javac`, `java`, and the classpath work, plus a `Makefile` (`make`, `make run`, `make clean`).

## Development

```sh
cargo build --release
cargo install --path .        # install locally
```

## Releasing

Push a tag and GitHub Actions builds binaries for Linux, macOS and Windows and attaches them to a GitHub Release:

```sh
git tag v0.3.2 && git push origin v0.3.2
```
