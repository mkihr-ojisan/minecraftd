#[derive(clap::Parser)]
pub struct Cli {
    /// The minimum port number to use for the backend servers.
    #[clap(long, default_value = "30001")]
    pub port_min: u16,
    /// The maximum port number to use for the backend servers.
    #[clap(long, default_value = "30100")]
    pub port_max: u16,
    /// The bind address for the proxy server.
    #[clap(long, default_value = "0.0.0.0:25565")]
    pub proxy_server_bind_address: String,
}
