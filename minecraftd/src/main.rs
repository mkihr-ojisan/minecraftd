use crate::{config::init_config, lock::Lock};

mod alert;
mod auto_start;
mod auto_update;
mod bridge;
mod config;
mod extension;
mod java_runtime;
mod lock;
mod metrics;
mod port_pool;
mod proxy_server;
mod runner;
mod server;
mod server_implementations;
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

    init_config().await?;

    port_pool::init();
    metrics::init().await?;
    proxy_server::init().await?;
    runner::init().await?;
    auto_update::init();

    socket::start_server().await?;

    runner::shutdown().await;
    metrics::shutdown().await;

    Ok(())
}
