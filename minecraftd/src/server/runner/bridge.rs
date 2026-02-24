use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::bail;
use bridge_protocol::{
    request::Payload as RequestPayload, response::Payload as ResponsePayload, *,
};
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
    sync::Mutex,
};
use uuid::Uuid;

use crate::server::runner::{RUNNER, ServerStatus, get_server_status};

const BRIDGE_CONNECT_RETRY_INTERVAL_SECS: u64 = 5;
const BRIDGE_CONNECT_MAX_RETRIES: u32 = 10;

#[derive(Debug)]
pub struct Bridge {
    stream: UnixStream,
}

impl Bridge {
    pub fn socket_path(server_dir: &Path) -> PathBuf {
        server_dir.join("minecraftd.sock")
    }

    pub async fn connect(server_dir: &Path) -> anyhow::Result<Self> {
        let socket_path = Self::socket_path(server_dir);
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream })
    }

    pub async fn send_request(&mut self, request: Request) -> anyhow::Result<Response> {
        let request_bytes = request.encode_to_vec();
        self.stream.write_u32(request_bytes.len() as u32).await?;
        self.stream.write_all(&request_bytes).await?;

        let response_len = self.stream.read_u32().await? as usize;
        let mut response_bytes = vec![0u8; response_len];
        self.stream.read_exact(&mut response_bytes).await?;
        let response = Response::decode(&response_bytes[..])?;

        if let Some(ResponsePayload::ErrorResponse(err)) = &response.payload {
            bail!("{}", err.message);
        }

        Ok(response)
    }

    pub async fn get_server_metrics(&mut self) -> anyhow::Result<ServerMetrics> {
        let request = Request {
            payload: Some(RequestPayload::GetServerMetricsRequest(
                GetServerMetricsRequest {},
            )),
        };
        let response = self.send_request(request).await?;
        match response.payload {
            Some(ResponsePayload::GetServerMetricsResponse(GetServerMetricsResponse {
                server_metrics: Some(server_metrics),
            })) => Ok(server_metrics),
            _ => bail!("Unexpected response payload"),
        }
    }
}

pub fn spawn_bridge_connector(id: Uuid, server_dir: PathBuf) {
    tokio::spawn(async move {
        for attempt in 1..=BRIDGE_CONNECT_MAX_RETRIES {
            if get_server_status(id).await != Some(ServerStatus::Ready) {
                debug!("Server is no longer ready, stopping bridge connector");
                break;
            }

            match Bridge::connect(&server_dir).await {
                Ok(bridge) => {
                    info!("Successfully connected to bridge");

                    let runner = RUNNER.lock().await;
                    let Some(server) = runner.running_servers.get(&id) else {
                        return;
                    };
                    server
                        .bridge
                        .set(Mutex::new(bridge))
                        .expect("bridge already set");

                    return;
                }
                Err(e) => {
                    debug!("Failed to connect to bridge on attempt {attempt}: {e:?}");
                    tokio::time::sleep(Duration::from_secs(BRIDGE_CONNECT_RETRY_INTERVAL_SECS))
                        .await;
                }
            }
        }
        warn!("Failed to connect to bridge");
    });
}
