# sunshine-stream-apps-manager

Modular CLI tool that integrates with [Sunshine](https://github.com/LizardByte/Sunshine) stream manager to read game libraries from multiple services and generate proper `apps.json` configuration.

## Features

- **Multi-platform scanning** — Steam, Epic Games Store, Lutris
- **Plugin architecture** — easily extendable with new game library sources
- **Remote scanning** — SSH/SFTP support for remote game libraries
- **Metadata fetching** — download posters from SteamGridDB
- **Backup & cleanup** — automatic backups before config changes
- **i18n** — translations for EN, RU, FR, IT, ES, EL, KA
- **Cross-platform** — Linux, macOS, Windows (amd64, arm64)

## Architecture

Built with SOLID principles and Rust best practices:

- **SRP** — each module has a single responsibility, split into focused files
- **OCP** — `Plugin` trait allows adding new scanners without modifying existing code
- **LSP** — all plugins implement `Plugin` trait and are interchangeable
- **DIP** — service layer depends on abstractions, not concrete implementations
- **Async I/O** — tokio runtime for all network and file operations

## Installation

```bash
cargo build --release
```

The binary will be in `target/release/sunshine-stream-apps-manager`.

## Usage

```bash
# Show default paths for each service
sunshine-stream-apps-manager show-paths

# Scan for installed games
sunshine-stream-apps-manager scan --plugins steam,egs

# Generate Sunshine config with metadata
sunshine-stream-apps-manager generate --fetch-metadata --steamgrid-key YOUR_API_KEY

# Remove all tool-managed apps
sunshine-stream-apps-manager cleanup

# Dry run (no changes written)
sunshine-stream-apps-manager generate --dry-run
```

## Configuration

Example configs are in the `examples/` directory. Copy the one that matches your setup and adjust paths/credentials:

| File | Description |
|------|-------------|
| `examples/config-local.yaml` | All services on the local machine |
| `examples/config-ssh.yaml` | Scan game libraries on a remote machine via SSH |

### Key fields

| Field | Description |
|-------|-------------|
| `steam_path` | Path to Steam library (with `transport: local` or `ssh`) |
| `egs_manifest_path` | EGS `InstalledGames.json` path |
| `egs_install_path` | EGS game install directory |
| `sunshine_config_dir` | Sunshine config directory (where `apps.json` lives) |
| `lutris_path` | Lutris config directory |
| `ssh` | SSH connection config (`host`, `port`, `user`, `key_path`) |
| `plugins` | List of plugins to scan (`steam`, `egs`, `lutris`) |
| `fetch_metadata` | Download posters from SteamGridDB |
| `steamgrid_api_key` | API key for SteamGridDB (get one at steamgriddb.com) |
| `language` | UI language (`en`, `ru`, `fr`, `it`, `es`, `el`, `ka`) |
| `dry_run` | Preview changes without writing files |
| `use_flat_spawn` | Use `flatpak-spawn --host` for Steam (Flatpak) |

### SSH transport

Set `transport: ssh` on any path field and provide the `ssh` block to scan
files on a remote machine. The `sunshine_config_dir` transport determines
where the generated `apps.json` is written.

## CLI Reference

| Command | Description |
|---------|-------------|
| `scan` | Scan for installed games from various sources |
| `generate` | Generate Sunshine config with discovered games |
| `cleanup` | Remove all tool-managed apps from Sunshine config |
| `version` | Print the version number |
| `show-paths` | Show default paths for each service |

Global options: `--config`, `--dry-run`, `--language`, `--verbose`, `--show-paths`

## CI/CD

GitHub Actions builds for all platforms on push/PR:

- Linux: `x86_64`, `aarch64`
- macOS: `x86_64`, `aarch64`
- Windows: `x86_64`, `aarch64`

Artifacts are uploaded as build artifacts for 7 days.

## License

MIT
