# acli

[![Documentation](https://docs.rs/acli/badge.svg)](https://ocasazza.github.io/acli/docs/index.html)

Atlassian CLI and TUI tools for engineers and administrators.


## Environment Setup

### NixOS/Nix (requires nixos-darwin or nixos installation)

```bash
direnv allow . && om --help
```

### Cargo (requires rustup and cargo)

```bash
cargo --help
```

## Configuration

Create `.env` file:
```
ATLASSIAN_URL="https://your-company.atlassian.net"
ATLASSIAN_USERNAME="your-email@company.com"
ATLASSIAN_API_TOKEN="your-api-token"
```

## Usage

```
cargo run --bin acli --help
```

or try the TUI

```
cargo run --bin atui
```

See [rustdocs](https://ocasazza.github.io/acli/docs/index.html) for full documentation.
