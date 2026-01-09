# Sapo CLI

Pre-install security scanner for npm packages. Scans packages **before** you install them.

[![CI](https://github.com/Salta1414/sapo-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Salta1414/sapo-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Salta1414/sapo-cli)](https://github.com/Salta1414/sapo-cli/releases)

## Installation

### macOS / Linux

```bash
curl -fsSL https://sapo.dev/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://sapo.dev/install.ps1 | iex
```

## How It Works

Sapo wraps your package manager (npm, pnpm, yarn, bun). When you install a package:

1. **Intercept** - Sapo catches the install command
2. **Scan** - Package is checked against our threat database
3. **Decide** - Safe packages install normally, risky ones show warnings

```
$ npm install lodash

  [>] Scanning: lodash@4.17.21
  [✓] Package is safe (score: 3)
  
  ... npm install continues ...
```

## Commands

| Command | Description |
|---------|-------------|
| `sapo status` | Show current status |
| `sapo scan <pkg>` | Scan a package without installing |
| `sapo toggle` | Enable/disable scanning |
| `sapo trust <pkg>` | Whitelist a package |
| `sapo login` | Link device to your account |
| `sapo upgrade` | Upgrade to Pro |
| `sapo update` | Update Sapo to latest version |
| `sapo uninstall` | Remove Sapo |

## What We Detect

- 🚫 Known malware from security advisories
- 🔤 Typosquatting (fake packages like `lodahs`, `reacct`)
- ⚡ Malicious install scripts
- 🔒 Credential access patterns
- 🌐 Suspicious network activity

## Pro Features

Upgrade to Pro for advanced protection:

- ML-based anomaly detection
- Behavioral sandbox analysis
- Runtime monitoring
- Dashboard & analytics

## Building from Source

```bash
# Clone
git clone https://github.com/Salta1414/sapo-cli.git
cd sapo-cli

# Build
cargo build --release

# Run
./target/release/sapo-cli --help
```

## License

MIT License - see [LICENSE](LICENSE)

## Links

- [Website](https://sapo.dev)
- [Documentation](https://sapo.dev/docs)
- [Changelog](https://sapo.dev/changelog)
