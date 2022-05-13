<div align="center">

![gobang](./resources/logo.png)

gobang is currently in alpha

A cross-platform TUI database management tool written in Rust

[![github workflow status](https://img.shields.io/github/workflow/status/TaKO8Ki/gobang/CI/main)](https://github.com/TaKO8Ki/gobang/actions) [![crates](https://img.shields.io/crates/v/gobang.svg?logo=rust)](https://crates.io/crates/gobang)

![gobang](./resources/gobang.gif)

</div>

## Features

- Cross-platform support (macOS, Windows, Linux)
- Multiple Database support (MySQL, PostgreSQL, SQLite)
- Intuitive keyboard only control

## TODOs

- [ ] SQL editor
- [ ] Custom key bindings
- [ ] Custom theme settings
- [ ] Support the other databases

## What does "gobang" come from?

gobang means a Japanese game played on goban, a go board. The appearance of goban looks like table structure. And I live in Kyoto, Japan. In Kyoto city, streets are laid out on a grid (We call it “goban no me no youna (碁盤の目のような)”). They are why I named this project "gobang".

## Installation

### With Homebrew (Linux, macOS)

If you’re using Homebrew or Linuxbrew, install the gobang formula:

```
brew install tako8ki/tap/gobang
```

### On Windows

If you're a Windows Scoop user, then you can install gobang from the [official bucket](https://github.com/ScoopInstaller/Main/blob/master/bucket/gobang.json):

```
scoop install gobang
```
### On NixOS

If you're a Nix user, you can install [gobang](https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/tools/database/gobang/default.nix) from nixpkgs:

```
$ nix-env --install gobang
```

### On NetBSD

If you're a NetBSD user, then you can install gobang from [pkgsrc](https://pkgsrc.se/databases/gobang):

```
pkgin install gobang
```

### With Cargo (Linux, macOS, Windows)

If you already have a Rust environment set up, you can use the `cargo install` command:

```
cargo install --version 0.1.0-alpha.5 gobang
```

### From binaries (Linux, macOS, Windows)

- Download the [latest release binary](https://github.com/TaKO8Ki/gobang/releases) for your system
- Set the `PATH` environment variable

## Usage

```
$ gobang
```

```
$ gobang -h
USAGE:
    gobang [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-path <config-path>    Set the config file
```

If you want to add connections, you need to edit your config file. For more information, please see [Configuration](#Configuration).

## Keymap

### Default keymap

| Key | Description |
| ---- | ---- |
| <kbd>h</kbd>, <kbd>j</kbd>, <kbd>k</kbd>, <kbd>l</kbd> | Scroll left/down/up/right |
| <kbd>Ctrl</kbd> + <kbd>u</kbd>, <kbd>Ctrl</kbd> + <kbd>d</kbd> | Scroll up/down multiple lines |
| <kbd>g</kbd> , <kbd>G</kbd> | Scroll to top/bottom |
| <kbd>H</kbd>, <kbd>J</kbd>, <kbd>K</kbd>, <kbd>L</kbd> | Extend selection by one cell left/down/up/right |
| <kbd>y</kbd> | Copy a cell value |
| <kbd>←</kbd>, <kbd>→</kbd> | Move focus to left/right |
| <kbd>c</kbd> | Move focus to connections |
| <kbd>/</kbd> | Filter |
| <kbd>?</kbd> | Help |
| <kbd>1</kbd>, <kbd>2</kbd>, <kbd>3</kbd>, <kbd>4</kbd>, <kbd>5</kbd> | Switch to records/columns/constraints/foreign keys/indexes tab |

### Custom keymap
The location of the file depends on your OS:

- macOS: `$HOME/.config/gobang/key_bind.ron`
- Linux: `$HOME/.config/gobang/key_bind.ron`
- Windows: `%APPDATA%/gobang/key_bind.ron`

A sample `key_bind.ron` is [here](https://github.com/TaKO8Ki/gobang/tree/main/examples/key_bind.ron).


## Configuration

The location of the file depends on your OS:

- macOS: `$HOME/.config/gobang/config.toml`
- Linux: `$HOME/.config/gobang/config.toml`
- Windows: `%APPDATA%/gobang/config.toml`

A sample `config.toml` file is [here](https://github.com/TaKO8Ki/gobang/tree/main/examples/config.toml)

## Contribution

Contributions, issues and pull requests are welcome!
