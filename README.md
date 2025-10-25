# ShootDPI

A simple cross-platform Rust launcher for running embedded `ciadpi-*` binaries with arguments.

## Features

- Embedded binaries for different architectures (`x86_64`, `aarch64`, `mips`, `i686`, `windows`)
- Built-in strategies + user strategies from `~/.shootdpi/strategies/`
- Automatic binary selection by OS and architecture
- Outputs stdout/stderr directly in the terminal
- Cross-platform (Linux, Windows)

## Installation

```bash
git clone https://github.com/akaruinekooff/shootdpi.git
cd shootdpi
cargo build --release
```

## Usage

```bash
cargo run
```

1. Choose a strategy from the menu
2. The program runs the corresponding binary with arguments from the strategy
3. Output is shown in the terminal

## Adding strategies

* Built-in: add a file to `strategies/` and rebuild
* User: add a file to `~/.shootdpi/strategies/`
* Filename = strategy name, file content = arguments for the binary