use crate::cli::Extensions;

mod add;

pub async fn extensions(command: Extensions) -> anyhow::Result<()> {
    match command {
        Extensions::Add(args) => {
            add::add(args).await?;
        }
    }

    Ok(())
}
