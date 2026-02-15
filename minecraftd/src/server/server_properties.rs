use std::{borrow::Cow, path::Path};

use anyhow::bail;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
};

#[derive(Debug, Default)]
pub struct ServerProperties {
    lines: Vec<Line>,
}

#[derive(Debug)]
enum Line {
    Property(String, String),
    Comment(String),
}

impl ServerProperties {
    pub async fn load(server_dir: &Path) -> anyhow::Result<Self> {
        let file = File::open(server_dir.join("server.properties")).await?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        let mut lines_stream = reader.lines();
        while let Some(line) = lines_stream.next_line().await? {
            if line.starts_with('#') {
                lines.push(Line::Comment(line));
            } else if let Some((key, value)) = line.split_once('=') {
                lines.push(Line::Property(key.to_string(), value.to_string()));
            } else {
                bail!("Invalid line in server.properties: {}", line);
            }
        }

        Ok(Self { lines })
    }

    pub async fn save(&self, server_dir: &Path) -> anyhow::Result<()> {
        let mut file = File::create(server_dir.join("server.properties")).await?;
        for line in &self.lines {
            match line {
                Line::Property(key, value) => {
                    file.write_all(key.as_bytes()).await?;
                    file.write_all(b"=").await?;
                    file.write_all(value.as_bytes()).await?;
                }
                Line::Comment(comment) => {
                    file.write_all(comment.as_bytes()).await?;
                }
            }
            file.write_all(b"\n").await?;
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        for line in &self.lines {
            if let Line::Property(k, v) = line
                && k == key
            {
                return Some(v);
            }
        }
        None
    }

    pub fn set<'a>(&mut self, key: impl Into<Cow<'a, str>>, value: impl Into<String>) {
        let key = key.into();

        for line in &mut self.lines {
            if let Line::Property(k, v) = line
                && k == &key
            {
                *v = value.into();
                return;
            }
        }
        self.lines
            .push(Line::Property(key.into_owned(), value.into()));
    }
}
