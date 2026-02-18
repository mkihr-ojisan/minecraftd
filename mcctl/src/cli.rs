use std::path::PathBuf;

#[derive(clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Create a new server
    Create(CreateArgs),
    /// Start a server
    Start(StartArgs),
    /// Stop a server
    Stop(StopArgs),
    /// Restart a server
    Restart(RestartArgs),
    /// Kill a server
    Kill(KillArgs),
    /// Attach to a server's console
    Attach(AttachArgs),
    /// Update a server to the latest version
    Update(UpdateArgs),
    /// List all running servers
    Ps,
    /// Manage server mods/plugins
    Extensions {
        #[command(subcommand)]
        command: Extensions,
    },
}

#[derive(clap::Args)]
pub struct CreateArgs {
    /// The directory to create the server in. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
    /// The name of the server.
    #[arg(short, long)]
    pub name: Option<String>,
    /// The server implementation to use.
    #[arg(short = 'i', long)]
    pub server_implementation: Option<String>,
    /// The version of the server.
    #[arg(short, long)]
    pub version: Option<String>,
    /// The build number of the server.
    #[arg(short, long)]
    pub build: Option<String>,
    /// The type of connection to use for the server. Possible values are "direct" and "proxy".
    #[arg(short, long)]
    pub connection: Option<String>,
    /// The hostname to use for the server if connection type is "proxy".
    #[arg(long)]
    pub hostname: Option<String>,
}

#[derive(clap::Args)]
pub struct StartArgs {
    /// The directory of the server to start. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct StopArgs {
    /// The directory of the server to stop. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct RestartArgs {
    /// The directory of the server to restart. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct KillArgs {
    /// The directory of the server to kill. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct AttachArgs {
    /// The directory of the server to attach to. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    /// The directory of the server to update. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
    /// The type of update to perform.
    #[clap(short, long, default_value = "stable")]
    pub update_type: UpdateType,
}

#[derive(Clone, clap::ValueEnum)]
pub enum UpdateType {
    /// Update to the latest stable version of the server.
    Stable,
    /// Update to the latest unstable version of the server (e.g. snapshots).
    Latest,
}

#[derive(clap::Subcommand)]
pub enum Extensions {
    /// Add a mod/plugin to the server
    Add(ExtensionsAddArgs),
}

#[derive(clap::Args)]
pub struct ExtensionsAddArgs {
    /// The directory of the server to add the mod/plugin to. If not specified, current directory will be used.
    #[arg(short = 'd', long)]
    pub server_dir: Option<PathBuf>,
    /// Allow adding mods that are incompatible with the server version.
    #[arg(long)]
    pub allow_incompatible_versions: bool,
    /// Enable auto-updates for the mod/plugin.
    #[arg(long)]
    pub auto_update: Option<bool>,
    /// The URL of the mod/plugin to add.
    /// Currently only Modrinth is supported.
    /// If not specified, you will be prompted to search for mods/plugins.
    pub url: Option<String>,
}
