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

### Private key (IdentityFile)

Attach a private key to a host so you can just `ssh <name>` instead of
`ssh -i <key> <name>`:

```bash
zoi add zino-pg deploy@10.0.0.5:2222 --private-key ~/.ssh/id_me
# or the ssh-style short flag
zoi edit zino-pg deploy@10.0.0.5:2222 -i ~/.ssh/id_me
```

This writes an `identityfile` entry into `~/.ssh/config`, so `ssh zino-pg`
picks up the key automatically. Omitting `--private-key` on `edit` keeps the
existing key untouched.

## License
This project is licensed under the [MIT license](license).

[license]: ./LICENSE
