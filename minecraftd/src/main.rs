use crate::{config::init_config, lock::Lock};

mod config;
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

    init_config().await?;

    port::init_port_pool();

    tokio::spawn(async move {
        if let Err(e) = server::proxy_server::start().await {
            error!("Proxy server error: {e:?}");
        }
    });

    server::runner::start_auto_start_servers().await;
    server::auto_update::start();

    socket::start_server().await?;

    server::runner::shutdown().await;

    Ok(())
}
