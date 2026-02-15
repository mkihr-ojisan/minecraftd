# minecraftd - Minecraft Server Manager

`minecraftd` is a local Minecraft server management daemon. It can create, start, and stop multiple Minecraft servers, and optionally provides a hostname-based proxy so you can serve multiple backends behind a single public port (default: 25565).

- Daemon: `minecraftd`
- Control CLI: `mcctl`

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
# Optional: pass daemon flags here (example)
# ExecStart=%h/.local/bin/minecraftd --port-min 30001 --port-max 30100 --proxy-server-bind-address 0.0.0.0:25565

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

- `--name` (display name)
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

Connect directly to the `server-port` in `server.properties`.

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
- `java_runtime`: auto-downloaded Java runtime or custom Java (see below)
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
