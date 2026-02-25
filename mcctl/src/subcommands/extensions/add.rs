use std::{fmt::Display, time::Duration};

use anyhow::{Context, bail};
use mcctl_protocol::{ExtensionInfo, ExtensionType, client::Client};
use minecraftd_manifest::ServerManifest;

use crate::cli::ExtensionsAddArgs;

pub async fn add(args: ExtensionsAddArgs) -> anyhow::Result<()> {
    let mut client = Client::connect()
        .await
        .context("Failed to connect to minecraftd")?;

    let server_dir = match args.server_dir {
        Some(p) => p,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    if !ServerManifest::manifest_path(&server_dir).exists() {
        bail!(
            "No server manifest found in '{}'. Are you sure this is a valid server directory?",
            server_dir.display()
        );
    }

    let manifest = ServerManifest::load(&server_dir)
        .await
        .context("Failed to load server manifest")?;

    let (provider, type_, extension) = if let Some(url) = args.url {
        let result = client.get_extension_id_by_url(&url).await?;

        let type_ = ExtensionType::try_from(result.r#type).context("Invalid extension type")?;

        (
            result.provider,
            type_,
            ExtensionInfo {
                id: result.extension_id,
                name: url,
            },
        )
    } else {
        let providers = client.get_extension_providers().await?;
        let provider = inquire::Select::new("Extension provider:", providers).prompt()?;

        let type_ = match inquire::Select::new("Extension type:", vec!["mod", "plugin"]).prompt()? {
            "mod" => mcctl_protocol::ExtensionType::Mod,
            "plugin" => mcctl_protocol::ExtensionType::Plugin,
            _ => bail!("Invalid extension type"),
        };

        let search_query = inquire::Text::new("Search query:").prompt()?;

        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message("Searching for extensions...");
        pb.enable_steady_tick(Duration::from_millis(100));

        let extensions = client
            .search_extension(
                &provider,
                type_,
                &manifest.version,
                &search_query,
                args.allow_incompatible_versions,
            )
            .await
            .context("Failed to search for extensions")?;

        pb.finish_and_clear();

        struct ExtensionDisplay(mcctl_protocol::ExtensionInfo);
        impl Display for ExtensionDisplay {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.name)
            }
        }

        let extension = inquire::Select::new(
            "Select a extension:",
            extensions.into_iter().map(ExtensionDisplay).collect(),
        )
        .prompt()?
        .0;

        (provider, type_, extension)
    };

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Fetching extension versions...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let extension_versions = client
        .get_extension_versions(
            &provider,
            type_,
            &manifest.version,
            &extension.id,
            args.allow_incompatible_versions,
        )
        .await
        .context("Failed to get extension versions")?;

    pb.finish_and_clear();

    struct ExtensionVersionDisplay(mcctl_protocol::ExtensionVersionInfo);
    impl Display for ExtensionVersionDisplay {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.0.is_stable {
                write!(f, "{}", self.0.version)
            } else {
                write!(f, "{} (unstable)", self.0.version)
            }
        }
    }

    let latest_stable_index = extension_versions
        .iter()
        .position(|v| v.is_stable)
        .unwrap_or(0);

    let extension_version = inquire::Select::new(
        "Select a extension version:",
        extension_versions
            .into_iter()
            .map(ExtensionVersionDisplay)
            .collect(),
    )
    .with_starting_cursor(latest_stable_index)
    .prompt()?;

    let auto_update = if let Some(auto_update) = args.auto_update {
        auto_update
    } else {
        inquire::Confirm::new("Enable auto-updates for this extension?")
            .with_default(false)
            .prompt()?
    };

    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Adding extension to server...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let server_dir = server_dir
        .canonicalize()
        .context("Failed to canonicalize path")?
        .to_str()
        .context("Path is not valid UTF-8")?
        .to_string();
    let result = client
        .add_extension(
            &server_dir,
            &provider,
            type_,
            &extension.id,
            &extension_version.0.id,
            args.allow_incompatible_versions,
            auto_update,
        )
        .await
        .context("Failed to add extension to server")?;

    pb.finish_with_message("Extension added successfully. Added extensions:");
    for extension in result.added_extensions {
        println!("  - {}", extension.name);
    }

    Ok(())
}
