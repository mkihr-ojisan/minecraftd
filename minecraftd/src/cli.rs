#[derive(clap::Parser)]
pub struct Cli {
    #[clap(long, default_value = "30001")]
    pub port_min: u16,
    #[clap(long, default_value = "30100")]
    pub port_max: u16,
    #[clap(long, default_value = "0.0.0.0:25565")]
    pub proxy_server_bind_address: String,
}
