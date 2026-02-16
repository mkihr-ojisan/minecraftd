use std::path::Path;

pub async fn is_accepted(server_dir: &Path) -> anyhow::Result<bool> {
    let eula_path = server_dir.join("eula.txt");
    if !eula_path.exists() {
        return Ok(false);
    }

    let contents = tokio::fs::read_to_string(eula_path).await?;
    for line in contents.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("eula=") {
            return Ok(value.trim().eq_ignore_ascii_case("true"));
        }
    }

    Ok(false)
}

pub async fn accept(server_dir: &Path) -> anyhow::Result<()> {
    let eula_path = server_dir.join("eula.txt");
    let contents = "eula=true\n";
    tokio::fs::write(eula_path, contents).await?;
    Ok(())
}
