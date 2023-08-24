# zoi
zoi is a command line tool that helps to manage your ssh connections.

[![Crates.io](https://img.shields.io/crates/v/zoi)](https://crates.io/crates/zoi)

## Installation

```bash
cargo install zoi
```

Or install manually:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

export PATH=$PATH:~/.cargo/bin

cargo build --release
cp target/release/zoi ~/.cargo/bin
```

## Usage

```bash
zoi -h

Usage: zoi [COMMAND]

Commands:
  list        List all hosts
  add         Add a host
  delete      Delete a host
  delete-all  Delete all hosts
  edit        Edit a host
  rename      Rename a host
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## License
This project is licensed under the [MIT license](license).

[license]: ./LICENSE
