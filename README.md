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
