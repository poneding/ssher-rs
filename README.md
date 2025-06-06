# ssher

English | [简体中文](./README_zh-CN.md)

ssher is an easy-to-use command line tool for connecting to remote servers.

![demo](docs/images/demo.gif)

## Installation

- Install with `cargo install`

```bash
cargo install ssher
```

- Install from binary

For MacOS or Linux:

```bash
curl -sSL https://github.com/poneding/ssher-rs/raw/master/install.sh | sh
```

For Windows:

Download the lastest executable from [Releases](https://github.com/poneding/ssher-rs/releases/latest) and add it to the PATH.

## Usage

![usage](docs/images/usage.png)

1. Select a server and connect

```bash
ssher
ssher -s <server>
```

2. Add a server

```bash
ssher add
```

3. Remove servers

```bash
# remove, rm
ssher rm
ssher rm <server_a> <server_b>
```

4. List servers

```bash
# list, ls
ssher ls
```

5. Rename a server

```bash
ssher rename
ssher rename <server_a>
```

6. Edit a server

```bash
ssher edit
ssher edit <server>
```

7. Import servers from ssh config file

```bash
ssher import

ssher import -c <ssh_config_file>
```

8. Check version

```bash
# version, v
ssher v
```

9. Help

```bash
ssher help
```

## Completions

```bash
# bash
source <(ssher completion bash)
source <(COMPLETE=bash ssher)

# zsh
source <(ssher completion zsh)
source <(COMPLETE=zsh ssher)

# fish
ssher completion fish | source
source (COMPLETE=fish ssher | psub)

# powershell
ssher completion powershell > ssher.ps1
. .\ssher.ps1
$env:COMPLETE = "powershell"
ssher | Out-String | Invoke-Expression
```

> You can add the command to your shell's profile e.g. `~/.bashrc` or `~/.zshrc` to enable completions for each session.

## Configuration

The configuration file is saved in the `~/.ssher.yaml` file.
