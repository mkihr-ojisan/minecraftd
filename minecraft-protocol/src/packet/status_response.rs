use std::io::Cursor;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{stream_ext::ReadExt, text_component::TextComponent, varint::ReadVarInt};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub version: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<TextComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Players>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modinfo: Option<ModInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forge_data: Option<ForgeData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub name: TextComponent,
    pub protocol: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample: Option<Vec<PlayerSample>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSample {
    pub name: TextComponent,
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInfo {
    #[serde(rename = "type")]
    pub type_: String,
    pub mod_list: Vec<Mod1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod1 {
    pub modid: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeData {
    pub mods: Vec<Mod2>,
    #[serde(
        serialize_with = "encode_forge_data_d",
        deserialize_with = "decode_forge_data_d"
    )]
    pub d: ForgeDataD,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod2 {
    pub mod_id: String,
    pub modmarker: String,
}

#[derive(Debug, Clone)]
pub struct ForgeDataD {
    pub mods: Vec<Mod3>,
}

#[derive(Debug, Clone)]
pub struct Mod3 {
    pub mod_id: String,
    pub mod_version: Mod3Version,
}

#[derive(Debug, Clone)]
pub enum Mod3Version {
    Version(String),
    ServerOnly,
}

fn encode_forge_data_d<S>(_: &ForgeDataD, _: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    unimplemented!()
}

fn decode_forge_data_d<'de, D>(deserializer: D) -> Result<ForgeDataD, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let encoded_str = String::deserialize(deserializer)?;

    // https://github.com/MinecraftForge/MinecraftForge/blob/fe616ffe2040e30230e0d174fb65a1bde54c6322/src/main/java/net/minecraftforge/network/ServerStatusPing.java#L286
    let mut utf16_iter = encoded_str.encode_utf16();
    let size0 = utf16_iter
        .next()
        .ok_or(Error::custom("Unexpected end of input"))? as u32;
    let size1 = utf16_iter
        .next()
        .ok_or(Error::custom("Unexpected end of input"))? as u32;
    let size = size0 | (size1 << 15);

    let mut buf = Vec::new();
    let mut buffer = 0u32;
    let mut bits_in_buf = 0;
    for c in utf16_iter {
        while bits_in_buf >= 8 {
            buf.push((buffer & 0xFF) as u8);
            buffer >>= 8;
            bits_in_buf -= 8;
        }

        buffer |= (c as u32) << bits_in_buf;
        bits_in_buf += 15;
    }

    while buf.len() < size as usize {
        buf.push((buffer & 0xFF) as u8);
        buffer >>= 8;
        // bits_in_buf -= 8;
    }

    // https://github.com/MinecraftForge/MinecraftForge/blob/fe616ffe2040e30230e0d174fb65a1bde54c6322/src/main/java/net/minecraftforge/network/ServerStatusPing.java#L209
    let mut cursor = Cursor::new(buf);

    let _truncated = cursor.read_boolean().map_err(Error::custom)?;
    let mods_size = cursor.read_unsigned_short().map_err(Error::custom)?;

    let mut mods = Vec::with_capacity(mods_size as usize);
    for _ in 0..mods_size {
        let channel_size_and_version_flag = cursor.read_varint().map_err(Error::custom)?;
        let channel_size = channel_size_and_version_flag.0 as u32 >> 1;
        let is_ignore_server_only = channel_size_and_version_flag.0 & 0b1 != 0;
        let mod_id = cursor.read_string().map_err(Error::custom)?;
        let mod_version = if is_ignore_server_only {
            Mod3Version::ServerOnly
        } else {
            Mod3Version::Version(cursor.read_string().map_err(Error::custom)?)
        };

        for _ in 0..channel_size {
            let _channel_name = cursor.read_string().map_err(Error::custom)?;
            let _channel_version = cursor.read_string().map_err(Error::custom)?; // 1.21ではvarintになっている説がある
            let _required_on_client = cursor.read_boolean().map_err(Error::custom)?;
        }

        mods.push(Mod3 {
            mod_id,
            mod_version,
        });
    }

    Ok(ForgeDataD { mods })
}

impl StatusResponse {
    pub fn iter_mods(&self) -> impl Iterator<Item = (&str, &str)> {
        self.modinfo
            .iter()
            .flat_map(|x| x.mod_list.iter().map(|x| (&*x.modid, &*x.version)))
            .chain(
                self.forge_data
                    .iter()
                    .flat_map(|x| x.mods.iter().map(|x| (&*x.mod_id, &*x.modmarker))),
            )
            .chain(self.forge_data.iter().flat_map(|x| {
                x.d.mods.iter().map(|x| {
                    (
                        &*x.mod_id,
                        match &x.mod_version {
                            Mod3Version::Version(x) => &**x,
                            Mod3Version::ServerOnly => "server only",
                        },
                    )
                })
            }))
    }
}
