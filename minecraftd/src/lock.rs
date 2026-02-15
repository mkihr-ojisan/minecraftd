use std::fs::File;

use anyhow::Context;
use fs2::FileExt;

pub struct Lock {
    file: File,
}

impl Lock {
    pub fn acquire() -> anyhow::Result<Self> {
        let lock_path = lock_path()?;

        let file = File::create(&lock_path)
            .with_context(|| format!("Failed to open lock file at {}", lock_path.display()))?;

        file.try_lock_exclusive().context(
            "Failed to acquire exclusive lock on lock file. Is another instance running?",
        )?;

        Ok(Lock { file })
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        if let Err(e) = self.file.unlock() {
            error!("Failed to release lock: {:?}", e);
        }
    }
}

fn lock_path() -> anyhow::Result<std::path::PathBuf> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .context("XDG_RUNTIME_DIR environment variable is not set")?;
    Ok(std::path::Path::new(&runtime_dir).join("minecraftd.lock"))
}
