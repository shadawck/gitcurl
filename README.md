# Gitcurl

[![Docs](https://docs.rs/gitcurl/badge.svg)](<https://docs.rs/gitcurl/>) [![crates.io](https://img.shields.io/crates/v/gitcurl.svg)](https://crates.io/crates/gitcurl)

Clone a git repository without the need of git to be installed on your system.

## Installation

### Cargo

```bash
cargo install gitcurl
```

Clone a git repository in the current folder.

```bash
gitcurl https://github.com/shadawck/gitcurl
# or 
gitcurl shadawck:gitcurl
```

Clone a specific branch of a git repository in the current folder.

```bash
gitcurl https://github.com/shadawck/gitcurl -b main
```

Fetch a zip of the repository.

```bash
gitcurl -z https://github.com/shadawck/gitcurl
```

Output with to a specific path

```bash
gitcurl -z https://github.com/shadawck/gitcurl -o /my/clone/path/myzip.zip
```

```bash
gitcurl https://github.com/shadawck/gitcurl -o /my/clone/path
```

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
