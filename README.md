# minecraftd - Minecraft Server Manager

`minecraftd` is a local Minecraft server management daemon. It can create, start, and stop multiple Minecraft servers, and optionally provides a hostname-based proxy so you can serve multiple backends behind a single public port (default: 25565).

- Daemon: `minecraftd`
- Control CLI: `mcctl`

## Key Features

- Create servers (Vanilla / Paper)
  - Auto-download/cache server jars
- Start / stop / restart / kill
  - Stop is done via RCON (`stop`) when possible
- Attach to the console (via PTY)
  - `mcctl attach` connects to the server’s stdio
- List running servers (`mcctl ps`)
  - Queries player counts via Server List Ping for servers in the Ready state
- Manage mods/plugins
  - `mcctl extensions add` adds mods/plugins and their dependencies
  - Optional auto-update for each extension
- Server auto-update
  - Periodically checks for updates and restarts when safe
- Built-in metrics collection + TUI
  - `mcctl stats` shows charts for TPS/MSPT/memory/CPU/player count/etc
- Optional proxy mode
  - Built-in TCP proxy routes by the hostname in the Minecraft handshake (“server address”)
  - Example: connect to `a.example.test` for server A, `b.example.test` for server B

## Requirements

- Linux
- Rust toolchain (`cargo`)
- Protocol Buffers compiler (`protoc`)
  - Debian/Ubuntu: `sudo apt-get install -y protobuf-compiler`
  - Arch Linux: `sudo pacman -S protobuf`
- systemd (optional, for user service)

## Installation

### Build

From the repository root:

```bash
cargo build --release
```

Build artifacts:

- `target/release/minecraftd`
- `target/release/mcctl`

### Install binaries

```bash
install -Dm755 target/release/minecraftd ~/.local/bin/minecraftd
install -Dm755 target/release/mcctl ~/.local/bin/mcctl
```

### Running with systemd (user service)

Create a unit file at `~/.config/systemd/user/minecraftd.service`:

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

Restart=on-failure
RestartSec=2

[Install]
WantedBy=default.target
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

Remove binaries:

```bash
rm -f ~/.local/bin/{minecraftd,mcctl}
```

Remove the auto-downloaded Java runtimes and other data:

```bash
rm -rf ~/.local/share/minecraftd
```

## Usage

### 1) Start `minecraftd`

You can run `minecraftd` either via systemd (recommended) or manually.

#### Option A: systemd

If you set up the systemd unit in the Installation section:

```bash
systemctl --user enable --now minecraftd
```

#### Option B: run manually

If you installed `minecraftd` to your `PATH`:

```bash
RUST_LOG=info minecraftd
```

### 2) Create a server

Example: create a Paper server in a directory:

```bash
mcctl create -d ~/mc/servers/paper-1
```

Common options:

- `--name` (display name)
- `--server-implementation` (`vanilla`, `paper`, or `custom`)
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
mcctl start -d ~/mc/servers/paper-1
mcctl ps
mcctl stop -d ~/mc/servers/paper-1
```

Restart / kill:

```bash
mcctl restart -d ~/mc/servers/paper-1
mcctl kill -d ~/mc/servers/paper-1
```

Note:

- On first start, `mcctl start` will prompt you to accept the Minecraft EULA and will write `eula.txt` into the server directory.

### 4) Attach to the console

```bash
mcctl attach -d ~/mc/servers/paper-1
```

To detach from the console, press `Ctrl+C`.

### 5) Connect from a Minecraft client

#### For `direct`

Connect directly to the `server-port` in `server.properties`.

#### For `proxy`

In the Minecraft client, set the “Server Address” to the hostname you configured at creation time (`--hostname`).
`minecraftd` listens on `proxy_server.bind_address` (default `0.0.0.0:25565`) and forwards the connection to the correct backend based on that hostname.

For local testing, adding an `/etc/hosts` entry is convenient:

```text
127.0.0.1 paper-1.local
```

### 6) Update a server

Update the server manifest to the latest stable version/build (downloads/caches the new server jar if needed):

```bash
mcctl update -d ~/mc/servers/paper-1
```

To update to the latest available (may include unstable/snapshots depending on implementation):

```bash
mcctl update -d ~/mc/servers/paper-1 --update-type latest
```

Note:

- `mcctl update` requires the server to be stopped.

## Server Directory Layout

After running `mcctl create`, the server directory will contain at least:

- `minecraftd.yaml` (minecraftd manifest)

On startup, `server.properties` is created/updated with:

- `enable-rcon=true`
- `rcon.port` (allocated from the port pool)
- `rcon.password` (generated randomly if not already set)
- `server-port` (allocated automatically when using `proxy` mode)

## `minecraftd.yaml` (Manifest)

`minecraftd.yaml` is YAML. Common fields:

- `name`: display name
- `server_implementation`: `vanilla`, `paper`, or `custom`
- `version` / `build`: the chosen version/build
- `command`: start command placeholders:
  - `${java}`: Java executable path
  - `${server_jar}`: server jar path for the chosen implementation/version/build
- `java_runtime`: auto-downloaded Java runtime or custom Java (see below)
- `connection`: `direct` / `proxy` (`proxy` requires `hostname`)
- `auto_start`: if `true`, the server directory is added to an auto-start list on start
- `restart_on_failure`: if `true`, restarts the server when it exits unexpectedly

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
  - "${server_jar}"
  - "nogui"
java_runtime:
  type: mojang
  name: java-runtime-delta
connection:
  type: proxy
  hostname: paper-1.local
auto_start: true
restart_on_failure: true
auto_update: false
extensions: []
```

Example (use a custom Java):

```yaml
java_runtime:
  type: custom
  java_home: /usr/lib/jvm/temurin-21-jdk
```

## Extensions (Mods / Plugins)

Currently the only supported provider is Modrinth.

Interactive add (search by keyword):

```bash
mcctl extensions add -d ~/mc/servers/paper-1
```

Add by URL (Modrinth URL):

```bash
mcctl extensions add -d ~/mc/servers/paper-1 https://modrinth.com/mod/sodium
```

Notes:

- `--allow-incompatible-versions` lets you pick versions that don't match the server version.
- If you enable extension auto-updates, the daemon will check and update them when server auto-update runs.

## Server Auto-Update

If `auto_update: true` is set in `minecraftd.yaml`, `minecraftd` periodically checks for:

- newer server version/build (stable by default)
- newer extension versions (for entries with `auto_update: true`)

When an update is available, it waits for players to log out (up to a configured timeout) and then restarts the server.

## Stats (`mcctl stats`)

`minecraftd` collects metrics into local storage and `mcctl stats` shows them in a terminal UI:

```bash
mcctl stats -d ~/mc/servers/paper-1
```

Keys:

- `q` or `Ctrl+C`: quit
- `+` / `-`: change time scale
- Arrow keys / PageUp / PageDown: scroll

## Configuration (`config.yaml`)

Config file path:

- `$XDG_CONFIG_HOME/minecraftd/config.yaml` (typically `~/.config/minecraftd/config.yaml`)

If the file is missing or invalid, defaults are used.

Minimal example:

```yaml
port:
  min: 30001
  max: 30100
proxy_server:
  bind_address: "0.0.0.0:25565"
```

Common knobs (optional):

- `auto_update.update_check_interval` (default: 24h)
- `metrics.collection_interval` (default: 1s)
- `metrics.storage_retention` (default: 30d)

## Data Locations

`minecraftd` stores caches and metrics under the XDG data directory:

- `$XDG_DATA_HOME/minecraftd` (typically `~/.local/share/minecraftd`)
  - `versions/`: cached server jars
  - `runtimes/`: auto-downloaded Java runtimes
  - `extensions/`: cached mods/plugins
  - `metrics/`: time-series storage for `mcctl stats`
