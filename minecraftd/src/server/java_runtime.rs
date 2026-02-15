use std::{borrow::Cow, path::PathBuf};

use anyhow::Context;
use mojang_piston_api::java_runtime::manifest::File;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::util::lazy_init_http_client::LazyInitHttpClient;

static CLIENT: LazyInitHttpClient = LazyInitHttpClient::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JavaRuntime {
    Mojang { name: String },
    Custom { java_home: PathBuf },
}

impl JavaRuntime {
    pub fn java_home(&self) -> PathBuf {
        match self {
            JavaRuntime::Mojang { name } => {
                let mut path = dirs::data_dir().expect("Failed to get data directory");
                path.push("minecraftd");
                path.push("runtimes");
                path.push(name);
                path
            }
            JavaRuntime::Custom { java_home } => java_home.clone(),
        }
    }

    pub fn java_path(&self) -> PathBuf {
        let mut java_home = self.java_home();
        java_home.push("bin");
        java_home.push("java");
        java_home
    }

    pub async fn prepare(&self) -> anyhow::Result<()> {
        match self {
            JavaRuntime::Mojang { name } => {
                if self.java_path().exists() {
                    return Ok(());
                }

                let java_runtimes = mojang_piston_api::java_runtime::get_all_java_runtimes()
                    .await?
                    .linux;

                let manifest = java_runtimes
                    .get(name)
                    .and_then(|r| r.first())
                    .with_context(|| format!("Java runtime '{}' not found", name))?
                    .manifest
                    .get()
                    .await?;

                let mut files = manifest.files.iter().collect::<Vec<_>>();
                files.sort_by_key(|(path, _)| *path);

                let dir = self.java_home();
                tokio::fs::create_dir_all(&dir)
                    .await
                    .context("Failed to create java runtime directory")?;

                for (path, file_info) in files {
                    let path = dir.join(path);

                    match file_info {
                        File::Directory => {
                            trace!("Creating directory: {}", path.display());

                            tokio::fs::create_dir_all(&path).await.with_context(|| {
                                format!("Failed to create directory '{}'", path.display())
                            })?;
                        }
                        File::File {
                            downloads,
                            executable,
                        } => {
                            trace!("Downloading file: {}", path.display());

                            tokio::fs::create_dir_all(path.parent().unwrap())
                                .await
                                .with_context(|| {
                                    format!(
                                        "Failed to create parent directory for '{}'",
                                        path.display()
                                    )
                                })?;

                            let (is_compressed, url, sha1, size) =
                                if let Some(lzma) = &downloads.lzma {
                                    (true, &lzma.url, &lzma.sha1, lzma.size)
                                } else if let Some(raw) = &downloads.raw {
                                    (false, &raw.url, &raw.sha1, raw.size)
                                } else {
                                    anyhow::bail!("No download available for file {:?}", path);
                                };

                            let bytes = CLIENT.get(url).send().await?.bytes().await?;

                            if bytes.len() != size as usize {
                                anyhow::bail!(
                                    "Downloaded file size does not match expected size for file {:?}",
                                    path
                                );
                            }

                            let downloaded_hash = Sha1::digest(&bytes);
                            let expected_hash = hex::decode(sha1)?;

                            if downloaded_hash[..] != expected_hash[..] {
                                anyhow::bail!(
                                    "Downloaded file hash does not match expected hash for file {:?}",
                                    path
                                );
                            }

                            let decompressed = if is_compressed {
                                Cow::Owned(
                                    lzma::decompress(&bytes).context("Failed to decompress")?,
                                )
                            } else {
                                Cow::Borrowed(&*bytes)
                            };

                            tokio::fs::write(&path, &decompressed).await?;

                            if *executable {
                                use std::os::unix::fs::PermissionsExt;

                                let mut permissions =
                                    tokio::fs::metadata(&path).await?.permissions();
                                permissions.set_mode(0o755);
                                tokio::fs::set_permissions(&path, permissions).await?;
                            }
                        }
                        File::Link { target } => {
                            let target = path.parent().unwrap().join(target);

                            trace!(
                                "Creating symlink: {} -> {}",
                                path.display(),
                                target.display()
                            );

                            std::os::unix::fs::symlink(target, &path)?;
                        }
                    }
                }

                Ok(())
            }
            JavaRuntime::Custom { .. } => Ok(()),
        }
    }
}
