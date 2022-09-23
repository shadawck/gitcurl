# Gitcurl

[![crates.io](https://img.shields.io/crates/v/gitcurl.svg)](https://crates.io/crates/gitcurl) [![Build](https://github.com/shadawck/gitcurl/actions/workflows/build.yaml/badge.svg)](https://github.com/shadawck/gitcurl/actions/workflows/build.yaml)

Clone a git repository without the need of git to be installed on your system.

## Installation

### Cargo

```bash
cargo install gitcurl
```

### Pre-compiled binaries

For each release, pre-compiled version are available : <https://github.com/shadawck/gitcurl/releases/latest/>

| OS             | Arch    |
| -------------- | ------- |
| linux          | x86\_64 |
| linux          | armv7   |
| linux          | arm64   |
| macos          | x86\_64 |
| macos          | arm64   |
| windows (msvc) | x86\_64 |
| windows (msvc) | i686    |

## How to use gitcurl

Clone a git repository in the current folder:

- On github:

    ```bash
    gitcurl https://github.com/shadawck/gitcurl
    # or 
    gitcurl github:shadawck:gitcurl
    ```

- Or on Gitlab:

    ```bash
    gitcurl https://gitlab.com/tezos/tezos
    # or 
    gitcurl gitlab:tezos:tezos
    ```

- Or on premise Gitlab:

    ```bash
    gitcurl https://gitlab.kitware.com/utils/rust-gitlab
    # or 
    gitcurl gitlab.kitware.com:utils:rust-gitlab
    ```

---

Clone a specific branch of a git repository in the current folder.

```bash
gitcurl https://github.com/shadawck/gitcurl -b main
```

---

Fetch a zip of the repository.

```bash
gitcurl -z https://github.com/shadawck/gitcurl
```

---

Output to a specific path.

```bash
gitcurl https://github.com/shadawck/gitcurl -z -o /my/clone/path/myzip.zip
gitcurl https://github.com/shadawck/gitcurl -o /my/clone/path
```

---

## Options

```bash
$ gitcurl --help
Clone git repository with curl

USAGE:
    gitcurl [OPTIONS] <URL>

ARGS:
    <URL>    Github link or just <user_name_name>:<repo_name>

OPTIONS:
    -b, --branch <branch>    Clone a specific branch of git repositiry
    -h, --help               Print help information
    -o, --output <PATH>      Path to save or decompress the zip archive
    -V, --version            Print version information
    -z, --only-zip           Only fetch the zipfile of the git repository without decompressing
shadawck@shadow:/opt/Projet/gitcurl$ 
```
