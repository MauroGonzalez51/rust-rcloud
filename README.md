# 🚀 rcloud

A powerful CLI wrapper for [rclone](https://rclone.org/) that simplifies cloud storage synchronization with support for hooks, tags, and automated workflows.

[![Release](https://img.shields.io/github/v/release/MauroGonzalez51/rust-rcloud)](https://github.com/MauroGonzalez51/rust-rcloud/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![SonarQube](https://github.com/MauroGonzalez51/rust-rcloud/actions/workflows/sonar.yml/badge.svg)](https://github.com/MauroGonzalez51/rust-rcloud/actions/workflows/sonar.yml)

---

## ✨ Features

- 🔄 **Bidirectional Sync**: Push and pull files/directories to/from cloud storage
- 🪝 **Hook System**: Apply transformations (compression, encryption, etc.) before syncing
- 🏷️ **Tag-Based Organization**: Group paths by tags for batch operations
- 🔒 **Transaction Safety**: Automatic rollback on configuration errors
- 📦 **Compression Support**: Built-in ZIP compression with exclusion patterns
- 🔍 **Hash Verification**: Skip unchanged content automatically
- 🎯 **Interactive CLI**: Intuitive prompts for configuration
- 📊 **Registry Management**: JSON-based configuration with file locking

---

## 📥 Installation

### Windows

#### Using PowerShell (Recommended)

```powershell
irm https://github.com/MauroGonzalez51/rust-rcloud/releases/latest/download/rcloud-installer.ps1 | iex
```

#### Using MSI Installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MauroGonzalez51/rust-rcloud/releases/latest/download/rcloud-installer.sh | sh
```

#### From Source

```bash
git clone https://github.com/MauroGonzalez51/rust-rcloud.git
cd rust-rcloud
cargo install --path .
```

---

## ✅ Prerequisites

rcloud drives the [`rclone`](https://rclone.org/downloads/) binary; it does not
talk to cloud providers directly. Before using rcloud:

1. Install `rclone` and make sure it is on your `PATH` (or point rcloud at it
   with `--rclone` / the `RCLONE_PATH` environment variable).
2. Configure your remotes in rclone itself (`rclone config`). rcloud references
   those remotes by name.

Verify your setup at any time:

```bash
rcloud configure
```

---

## 🚀 Quick Start

```bash
# 1. Register a remote you already configured in rclone (name must match).
rcloud remote add --name mydrive --provider drive

# 2. Map a local path to a remote path (prompts for hooks and tags).
rcloud path add --remote-id <REMOTE_ID> \
  --local-path ~/Documents/notes \
  --remote-path backups/notes

# 3. Push it.
rcloud sync path <PATH_ID> --direction push

# 4. Pull it back somewhere else.
rcloud sync path <PATH_ID> --direction pull
```

Run `rcloud` with no arguments to open the interactive TUI, which exposes the
same operations as a navigable menu.

---

## 📖 Commands

Every command works both non-interactively (with flags) and interactively
(prompting for anything omitted).

### Remotes

```bash
rcloud remote list                        # list configured remotes
rcloud remote add --name <N> --provider <P>
rcloud remote update --id <ID> [--name <N>] [--provider <P>]
rcloud remote remove --id <ID>
rcloud remote ls <REMOTE_PATH>            # e.g. mydrive:documents
rcloud remote ls --path-config <PATH_ID> # resolve the path from the registry
```

### Paths

```bash
rcloud path list
rcloud path add --remote-id <ID> --local-path <PATH> --remote-path <PATH>
rcloud path remove --id <ID>
```

### Sync

```bash
rcloud sync path <PATH_ID> --direction <push|pull> [-F|--force] [-C|--clean]
rcloud sync all --tags tag1,tag2         # sync every path matching the tags
rcloud sync all                          # sync every configured path
```

- `--force` / `-F`: sync even if the content hash is unchanged.
- `--clean` / `-C`: remove the local destination before a pull.

### Other

```bash
rcloud configure                 # check rclone and initialize the registry
rcloud completion <shell>        # generate shell completions (bash, zsh, ...)
```

### Global options

Available on every subcommand:

| Flag                | Description                                         |
| ------------------- | --------------------------------------------------- |
| `-v`                | Increase verbosity (repeatable)                     |
| `-d`, `--debug`     | Enable debug logging                                |
| `-c`, `--config`    | Path to the config file (`.toml`)                   |
| `-r`, `--registry`  | Path to the registry file (`.json`)                 |
| `--rclone`          | Path to the `rclone` binary (`RCLONE_PATH` env var) |

---

## ⚙️ Configuration

rcloud keeps two files under your platform config directory by default (override
with `--config` / `--registry`):

- **`rcloud.toml`** — application settings: temp directory and TUI key
  bindings. Created from a built-in default on first run.
- **`registry.json`** — your remotes and path mappings. Reads are file-locked
  and writes are transactional (a failed write rolls back cleanly).

Example `rcloud.toml`:

```toml
[core]
# Directory for intermediate files created by hooks.
# Defaults to the system temp directory when unset.
# temp_path = "/tmp/rcloud"

[tui.keys]
quit  = ['q']
up    = ['k']
down  = ['j']
left  = ['h']   # go back / to parent
right = ['l']   # enter submenu / execute
```

---

## 🪝 Hooks

Hooks transform content around a transfer. On push they run in order before
upload; on pull they run in reverse order after download.

| Hook           | On push                     | On pull      |
| -------------- | --------------------------- | ------------ |
| **Zip**        | Zstd-compress into archive  | Extract      |
| **Encryption** | AES-256-GCM encrypt         | Decrypt      |
| **Backup**     | Rotated local/remote copies | Rotated copy |

Hooks are attached to a path when you run `rcloud path add`. To add your own
hook type, see [`docs/HOOKS.md`](docs/HOOKS.md).

Encrypted files use a `MARKER | SALT | NONCE | CIPHERTEXT` layout with a random
per-file salt, so the same password derives a distinct key per file.

---

## 🏗️ Architecture

For the crate layout, request flow, sync engine, and hook pipeline, see
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## 🛠️ Development

```bash
cargo build            # build
cargo test             # run unit + integration tests
cargo clippy           # lint
cargo doc --no-deps    # generate API docs
```

---

## 📄 License

Licensed under the [MIT License](https://opensource.org/licenses/MIT).

