use std::collections::HashMap;

use anyhow::Context;
use reqwest::Client;
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, Visitor},
};

#[derive(Deserialize)]
pub struct GetProjectResponse {
    #[serde(deserialize_with = "deserialize_map_as_vec")]
    pub versions: Vec<(String, Vec<String>)>,
}

fn deserialize_map_as_vec<'de, D, T>(deserializer: D) -> Result<Vec<(String, T)>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct VecVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for VecVisitor<T> {
        type Value = Vec<(String, T)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Vec<(String, T)>, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some((key, value)) = map.next_entry()? {
                vec.push((key, value));
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_map(VecVisitor(std::marker::PhantomData))
}

pub async fn get_project(client: &Client, project: &str) -> anyhow::Result<GetProjectResponse> {
    let url = format!("https://fill.papermc.io/v3/projects/{project}");
    let resp = client
        .get(url)
        .send()
        .await
        .context("Failed to send request to PaperMC API")?;

    let project: GetProjectResponse = resp
        .json()
        .await
        .context("Failed to parse PaperMC project response")?;

    Ok(project)
}

#[derive(Deserialize)]
pub struct Build {
    pub id: u32,
    pub channel: BuildChannel,
    pub downloads: HashMap<String, Download>,
}

#[derive(PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BuildChannel {
    Alpha,
    Beta,
    Stable,
    Recommended,
}

#[derive(Deserialize)]
pub struct Download {
    pub checksums: Checksums,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Checksums {
    pub sha256: String,
}

pub async fn get_builds(
    client: &Client,
    project: &str,
    version: &str,
) -> anyhow::Result<Vec<Build>> {
    let url = format!(
        "https://fill.papermc.io/v3/projects/{project}/versions/{version}/builds?channel=ALPHA&channel=BETA&channel=STABLE&channel=RECOMMENDED"
    );
    let resp = client
        .get(url)
        .send()
        .await
        .context("Failed to send request to PaperMC API")?;

    let builds = resp
        .json::<Vec<Build>>()
        .await
        .context("Failed to parse PaperMC builds response")?;

    Ok(builds)
}

pub async fn get_build(
    client: &Client,
    project: &str,
    version: &str,
    build: u32,
) -> anyhow::Result<Build> {
    let url =
        format!("https://fill.papermc.io/v3/projects/{project}/versions/{version}/builds/{build}");
    let resp = client
        .get(url)
        .send()
        .await
        .context("Failed to send request to PaperMC API")?;

    let build = resp
        .json::<Build>()
        .await
        .context("Failed to parse PaperMC build response")?;

    Ok(build)
}
