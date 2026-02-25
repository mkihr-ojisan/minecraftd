use clap::Parser;

use crate::cli::Subcommand;

mod cli;
mod eula;
mod subcommands;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = start().await {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    match args.command {
        Subcommand::Create(args) => {
            subcommands::create::create(args).await?;
        }
        Subcommand::Start(args) => {
            subcommands::start::start(args).await?;
        }
        Subcommand::Stop(args) => {
            subcommands::stop::stop(args).await?;
        }
        Subcommand::Restart(args) => {
            subcommands::restart::restart(args).await?;
        }
        Subcommand::Kill(args) => {
            subcommands::kill::kill(args).await?;
        }
        Subcommand::Attach(args) => {
            subcommands::attach::attach(args).await?;
        }
        Subcommand::Update(args) => {
            subcommands::update::update(args).await?;
        }
        Subcommand::Ps => {
            subcommands::ps::ps().await?;
        }
        Subcommand::Extensions { command } => {
            subcommands::extensions::extensions(command).await?;
        }
    }

    Ok(())
}
