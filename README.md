# minecraftd - Minecraft Server Manager

`minecraftd` is a local Minecraft server management daemon. It can create, start, and stop multiple Minecraft servers, and optionally provides a hostname-based proxy so you can serve multiple backends behind a single public port (default: 25565).

- Daemon: `minecraftd`
- Control CLI: `mcctl`

## Table of Contents

- [Key Features](#key-features)
- [Requirements](#requirements)
  - [Paths Used (typical)](#paths-used-typical)
- [Installation](#installation)
  - [Build locally](#build-locally)
  - [Install binaries](#install-binaries)
  - [Running with systemd (user service)](#running-with-systemd-user-service)
  - [Uninstall](#uninstall)
- [Usage (Quick Start)](#usage-quick-start)
- [Server Directory Layout](#server-directory-layout)
- [`minecraftd.yaml` (Configuration)](#minecraftdyaml-configuration)
- [Troubleshooting](#troubleshooting)

## Key Features

- Create servers (Vanilla / Paper)
  - Auto-download server jars
- Start / stop / restart / kill
  - Stop is done via RCON (`stop`) when possible
- Attach to the console (via PTY)
  - `mcctl attach` connects to the server’s stdio
- List running servers (`mcctl ps`)
  - Queries player counts via Server List Ping for servers in the Ready state
- Optional proxy mode
  - Built-in TCP proxy routes by the hostname in the Minecraft handshake (“server address”)
  - Example: connect to `a.example.test` for server A, `b.example.test` for server B

## Requirements

- Linux
- Rust toolchain (`cargo`)
- `protoc` (Protocol Buffers compiler)
  - Debian/Ubuntu: `sudo apt-get install -y protobuf-compiler`
  - Arch Linux: `sudo pacman -S protobuf`
- systemd (optional, for user service)

### Paths Used (typical)

- Daemon socket: `$XDG_RUNTIME_DIR/minecraftd.sock` ($XDG_RUNTIME_DIR is usually something like `/run/user/1000`)
- Daemon lock: `$XDG_RUNTIME_DIR/minecraftd.lock`
- Mojang Java Runtime (auto-download): typically `~/.local/share/minecraftd/runtimes/...`

## Installation

### Build locally

From the repository root:

```bash
cargo build --release
```

Build artifacts:

- `target/release/minecraftd`
- `target/release/mcctl`

### Install binaries

#### Option A: copy from `target/release` to `~/.local/bin` (recommended for systemd)

The systemd user unit below uses `ExecStart=%h/.local/bin/minecraftd`, so install the binaries to `~/.local/bin`:

```bash
install -Dm755 target/release/minecraftd ~/.local/bin/minecraftd
install -Dm755 target/release/mcctl ~/.local/bin/mcctl
```

#### Option B: `cargo install` (installs to `~/.cargo/bin`)

If you prefer `cargo install`, install the workspace packages directly:

```bash
cargo install --path minecraftd --bin minecraftd
cargo install --path mcctl --bin mcctl
```

If you use this option, either:

- change the systemd unit `ExecStart` to `%h/.cargo/bin/minecraftd`, or
- copy/symlink the binaries into `~/.local/bin`.

If `~/.local/bin` is not in your `PATH`, add it (for interactive shells):

```bash
export PATH="$HOME/.local/bin:$PATH"
```

To make this persistent, add the line to your shell startup file (e.g., `~/.profile`, `~/.bashrc`, or `~/.zshrc`).

### Running with systemd (user service)

1) Create a unit file at `~/.config/systemd/user/minecraftd.service`:

```ini
[Unit]
Description=Minecraft Server Manager (minecraftd)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=%h/.local/bin/minecraftd
# Optional: default logging verbosity
Environment=RUST_LOG=info
# Optional: pass daemon flags here (example)
# ExecStart=%h/.local/bin/minecraftd --port-min 30001 --port-max 30100 --proxy-server-bind-address 0.0.0.0:25565

Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
```

2) Reload and start it:

```bash
systemctl --user daemon-reload
systemctl --user enable --now minecraftd
```

3) Check status / logs:

```bash
systemctl --user status minecraftd
journalctl --user -u minecraftd -f
```

Notes:

- If you want `minecraftd` to keep running even when you are not logged in, enable lingering for your user:

```bash
sudo loginctl enable-linger "$USER"
```

- `mcctl` connects via the user runtime socket (typically `$XDG_RUNTIME_DIR/minecraftd.sock`), so make sure you run `mcctl` as the same user that runs the service.

### Uninstall

If you installed the systemd unit, disable and remove it:

```bash
systemctl --user disable --now minecraftd
rm -f ~/.config/systemd/user/minecraftd.service
systemctl --user daemon-reload
```

Remove binaries (pick the location you used):

```bash
rm -f ~/.local/bin/minecraftd ~/.local/bin/mcctl
rm -f ~/.cargo/bin/minecraftd ~/.cargo/bin/mcctl
```

Remove the auto-downloaded Mojang Java runtime cache (optional):

```bash
rm -rf ~/.local/share/minecraftd/runtimes
```

This only removes cached Java runtimes. Your server directories (worlds, configs, logs) are not affected.

## Usage (Quick Start)

Typical workflow: start the daemon, create a server, then `start` / `stop` / `attach` using `mcctl`.

### 1) Start `minecraftd`

You can run `minecraftd` either via systemd (recommended) or manually.

#### Option A: systemd (recommended)

If you set up the systemd unit in the Installation section:

```bash
systemctl --user enable --now minecraftd
```

Logs:

```bash
journalctl --user -u minecraftd -f
```

#### Option B: run manually (quick testing)

If you installed `minecraftd` to your `PATH`:

```bash
RUST_LOG=info minecraftd
```

Or, from the repository root (build artifacts):

```bash
RUST_LOG=info ./target/release/minecraftd
```

If `./target/release/minecraftd` does not exist yet, build it first:

```bash
cargo build --release
```

For up-to-date daemon options, run `minecraftd --help`. Common options:

- `--port-min` (default `30001`)
- `--port-max` (default `30100`)
  - Used to allocate backend ports for proxy mode and RCON ports
- `--proxy-server-bind-address` (default `0.0.0.0:25565`)
  - The public entrypoint for Minecraft clients (the proxy listener)

### 2) Create a server

Example: create a Paper server in a directory:

```bash
mcctl create ~/mc/servers/paper-1
```

Common options:

- `--server-implementation` (`vanilla` or `paper`)
- `--version` (e.g., `1.21.11`)
- `--build` (Paper build number, e.g., `123`)
- `--connection` (`direct` or `proxy`)
- `--hostname` (required for `proxy` connection mode, e.g., `paper-1.local`)

If you omit options, you will be prompted interactively.

Connection modes:

- `direct`: the server listens on its own `server-port` (in `server.properties`)
- `proxy`: clients connect to the `minecraftd` proxy (25565 by default). Routing is done by hostname (`--hostname`).

If you want to run multiple servers at once, `proxy` is usually the easiest option (backend ports are allocated automatically within the configured range).

### 3) Start / stop

```bash
mcctl start ~/mc/servers/paper-1
mcctl ps
mcctl stop ~/mc/servers/paper-1
```

Restart / kill:

```bash
mcctl restart ~/mc/servers/paper-1
mcctl kill ~/mc/servers/paper-1
```

### 4) Attach to the console

```bash
mcctl attach ~/mc/servers/paper-1
```

To detach from the console, press `Ctrl+C`.

### 5) Connect from a Minecraft client

#### For `direct`

Connect directly to the `server-port` in `server.properties` (default is usually 25565 unless changed).

#### For `proxy`

In the Minecraft client, set the “Server Address” to the hostname you configured at creation time (`--hostname`).
`minecraftd` listens on `--proxy-server-bind-address` (default `0.0.0.0:25565`) and forwards the connection to the correct backend based on that hostname.

For local testing, adding an `/etc/hosts` entry is convenient:

```text
127.0.0.1 paper-1.local
```

## Server Directory Layout

After running `mcctl create`, the server directory will contain at least:

- `server.jar`
- `eula.txt` (with `eula=true`)
- `minecraftd.yaml` (minecraftd config)

On startup, `server.properties` is created/updated with:

- `enable-rcon=true`
- `rcon.port` (allocated from the port pool)
- `rcon.password` (generated randomly if not already set)
- `server-port` (allocated automatically when using `proxy` mode)

## `minecraftd.yaml` (Configuration)

`minecraftd.yaml` is YAML. Common fields:

- `name`: display name
- `server_implementation`: `vanilla` or `paper`
- `version` / `build`: the chosen version/build
- `command`: start command (`${java}` will be replaced with the Java executable)
- `java_runtime`: Mojang runtime auto-download or a custom Java
- `connection`: `direct` / `proxy` (`proxy` requires `hostname`)

Example (proxy + Mojang runtime):

```yaml
name: paper-1
server_implementation: paper
version: "1.21.4"
build: "123"
command:
  - "${java}"
  - "-Xmx4G"
  - "-jar"
  - "server.jar"
  - "nogui"
java_runtime:
  type: mojang
  name: java-runtime-delta
connection:
  type: proxy
  hostname: paper-1.local
```

Example (use a custom Java):

```yaml
java_runtime:
  type: custom
  java_home: /usr/lib/jvm/temurin-21-jdk
```

## Troubleshooting

### `XDG_RUNTIME_DIR environment variable is not set`

`minecraftd` uses `XDG_RUNTIME_DIR` for its lock and socket. Under a systemd user session this is usually set automatically.
If it is not set, create an appropriate runtime directory and set `XDG_RUNTIME_DIR` (e.g., a directory only your user can read).

### `Failed to acquire exclusive lock ... Is another instance running?`

Another `minecraftd` instance is running (lock file: `$XDG_RUNTIME_DIR/minecraftd.lock`).

### `Failed to connect to minecraftd`

- Check that `minecraftd` is running
- Check that the socket exists: `$XDG_RUNTIME_DIR/minecraftd.sock`
- Check that you are using the same user (permissions on the runtime directory/socket matter)
