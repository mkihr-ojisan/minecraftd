use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use minecraftd_manifest::Connection;
use uuid::Uuid;

use crate::server::runner::RunningServer;

pub struct RunningServers {
    servers: HashMap<Uuid, RunningServer>,
    hostname_to_id: HashMap<String, Uuid>,
    server_dir_to_id: HashMap<PathBuf, Uuid>,
}

impl RunningServers {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            hostname_to_id: HashMap::new(),
            server_dir_to_id: HashMap::new(),
        }
    }

    pub fn insert(&mut self, server: RunningServer) {
        if let Connection::Proxy { hostname } = &server.manifest.connection {
            self.hostname_to_id.insert(hostname.to_string(), server.id);
        }
        self.server_dir_to_id
            .insert(server.server_dir.clone(), server.id);
        self.servers.insert(server.id, server);
    }

    pub fn remove(&mut self, id: &Uuid) -> Option<RunningServer> {
        let server = self.servers.remove(id)?;
        if let Connection::Proxy { hostname } = &server.manifest.connection {
            self.hostname_to_id.remove(hostname);
        }
        self.server_dir_to_id.remove(&server.server_dir);
        Some(server)
    }

    pub fn contains(&self, id: &Uuid) -> bool {
        self.servers.contains_key(id)
    }

    pub fn get(&self, id: &Uuid) -> Option<&RunningServer> {
        self.servers.get(id)
    }

    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut RunningServer> {
        self.servers.get_mut(id)
    }

    pub fn get_id_by_hostname(&self, hostname: &str) -> Option<Uuid> {
        self.hostname_to_id.get(hostname).copied()
    }

    pub fn get_id_by_server_dir(&self, server_dir: &Path) -> anyhow::Result<Option<Uuid>> {
        Ok(self
            .server_dir_to_id
            .get(&server_dir.canonicalize()?)
            .copied())
    }

    pub fn get_by_server_dir(&self, server_dir: &Path) -> anyhow::Result<Option<&RunningServer>> {
        let id = match self.get_id_by_server_dir(server_dir)? {
            Some(id) => id,
            None => return Ok(None),
        };
        Ok(self.servers.get(&id))
    }

    pub fn get_mut_by_server_dir(&mut self, server_dir: &Path) -> Option<&mut RunningServer> {
        let id = self.get_id_by_server_dir(server_dir).ok().flatten()?;
        self.servers.get_mut(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RunningServer> {
        self.servers.values()
    }
}
