use clap::Parser;

use crate::lock::Lock;

mod cli;
mod lock;
mod port;
mod server;
mod socket;
mod util;

#[macro_use]
extern crate log;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    pretty_env_logger::init();

    if let Err(e) = start().await {
        error!("{e:?}");
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    let _lock = Lock::acquire()?;

    let args = cli::Cli::parse();

    port::init_port_pool(args.port_min, args.port_max);

    tokio::spawn(async move {
        if let Err(e) = server::proxy_server::start(&args.proxy_server_bind_address).await {
            error!("Proxy server error: {e:?}");
        }
    });

    socket::start_server().await?;

    server::runner::shutdown().await;

    Ok(())
}
