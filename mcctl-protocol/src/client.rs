use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
};

use crate::{
    error::Error, request::Payload as RequestPayload, response::Payload as ResponsePayload,
    terminal_input::Input, *,
};

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub async fn connect() -> Result<Self, Error> {
        let socket_path = socket_path()?;
        let stream = UnixStream::connect(socket_path).await?;

        Ok(Client { stream })
    }

    pub async fn send_request(
        &mut self,
        request_payload: RequestPayload,
    ) -> Result<Option<ResponsePayload>, Error> {
        let mut buffer = Vec::new();
        request_payload.encode(&mut buffer);
        self.stream.write_u32(buffer.len() as u32).await?;
        self.stream.write_all(&buffer).await?;

        let response_length = self.stream.read_u32().await?;
        let mut response_bytes = vec![0u8; response_length as usize];
        self.stream.read_exact(&mut response_bytes).await?;

        let response = Response::decode(&response_bytes[..])?;

        match response.payload {
            Some(ResponsePayload::ErrorResponse(ErrorResponse { message })) => {
                Err(Error::ErrorResponse { message })
            }
            _ => Ok(response.payload),
        }
    }

    pub async fn get_server_implementations(&mut self) -> Result<Vec<String>, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetServerImplementationsRequest(
                GetServerImplementationsRequest {},
            ))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetServerImplementationsResponse(
                GetServerImplementationsResponse { implementations },
            )) => Ok(implementations),
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetServerImplementationsResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn get_versions(
        &mut self,
        server_implementation: impl Into<String>,
    ) -> Result<Vec<Version>, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetVersionsRequest(GetVersionsRequest {
                server_implementation: server_implementation.into(),
            }))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetVersionsResponse(GetVersionsResponse { versions })) => {
                Ok(versions)
            }
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetVersionsResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn get_builds(
        &mut self,
        server_implementation: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<Vec<Build>, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetBuildsRequest(GetBuildsRequest {
                server_implementation: server_implementation.into(),
                version: version.into(),
            }))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetBuildsResponse(GetBuildsResponse { builds })) => Ok(builds),
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetBuildsResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_server(
        &mut self,
        server_dir: impl Into<String>,
        name: impl Into<String>,
        server_implementation: impl Into<String>,
        version: impl Into<String>,
        build: impl Into<String>,
        connection: ConnectionType,
        hostname: Option<String>,
    ) -> Result<(), Error> {
        let response_payload = self
            .send_request(RequestPayload::CreateServerRequest(CreateServerRequest {
                server_dir: server_dir.into(),
                name: name.into(),
                server_implementation: server_implementation.into(),
                version: version.into(),
                build: build.into(),
                connection: connection as i32,
                hostname,
            }))
            .await?;

        match response_payload {
            None => Ok(()),
            _ => Err(Error::UnexpectedResponseType {
                expected: "CreateServerResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn start_server(&mut self, server_dir: impl Into<String>) -> Result<(), Error> {
        let response_payload = self
            .send_request(RequestPayload::StartServerRequest(StartServerRequest {
                server_dir: server_dir.into(),
            }))
            .await?;

        match response_payload {
            None => Ok(()),
            _ => Err(Error::UnexpectedResponseType {
                expected: "StartServerResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn stop_server(&mut self, server_dir: impl Into<String>) -> Result<(), Error> {
        let response_payload = self
            .send_request(RequestPayload::StopServerRequest(StopServerRequest {
                server_dir: server_dir.into(),
            }))
            .await?;

        match response_payload {
            None => Ok(()),
            _ => Err(Error::UnexpectedResponseType {
                expected: "StopServerResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn kill_server(&mut self, server_dir: impl Into<String>) -> Result<(), Error> {
        let response_payload = self
            .send_request(RequestPayload::KillServerRequest(KillServerRequest {
                server_dir: server_dir.into(),
            }))
            .await?;

        match response_payload {
            None => Ok(()),
            _ => Err(Error::UnexpectedResponseType {
                expected: "KillServerResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn attach_terminal(
        mut self,
        server_dir: impl Into<String>,
    ) -> Result<(TerminalReader, TerminalWriter), Error> {
        let response_payload = self
            .send_request(RequestPayload::AttachTerminalRequest(
                AttachTerminalRequest {
                    server_dir: server_dir.into(),
                },
            ))
            .await?;

        let (stream_reader, stream_writer) = self.stream.into_split();

        match response_payload {
            None => Ok((
                TerminalReader { stream_reader },
                TerminalWriter { stream_writer },
            )),
            _ => Err(Error::UnexpectedResponseType {
                expected: "AttachTerminalResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn get_running_servers(&mut self) -> Result<Vec<RunningServer>, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetRunningServersRequest(
                GetRunningServersRequest {},
            ))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetRunningServersResponse(GetRunningServersResponse {
                servers,
            })) => Ok(servers),
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetRunningServersResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn wait_server_ready(&mut self, server_dir: impl Into<String>) -> Result<(), Error> {
        let response_payload = self
            .send_request(RequestPayload::WaitServerReadyRequest(
                WaitServerReadyRequest {
                    server_dir: server_dir.into(),
                },
            ))
            .await?;

        match response_payload {
            None => Ok(()),
            _ => Err(Error::UnexpectedResponseType {
                expected: "WaitServerReadyResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn restart_server(&mut self, server_dir: impl Into<String>) -> Result<(), Error> {
        let response_payload = self
            .send_request(RequestPayload::RestartServerRequest(RestartServerRequest {
                server_dir: server_dir.into(),
            }))
            .await?;

        match response_payload {
            None => Ok(()),
            _ => Err(Error::UnexpectedResponseType {
                expected: "RestartServerResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn update_server(
        &mut self,
        server_dir: impl Into<String>,
        update_type: UpdateType,
    ) -> Result<UpdateServerResponse, Error> {
        let response_payload = self
            .send_request(RequestPayload::UpdateServerRequest(UpdateServerRequest {
                server_dir: server_dir.into(),
                update_type: update_type as i32,
            }))
            .await?;

        match response_payload {
            Some(ResponsePayload::UpdateServerResponse(update_result)) => Ok(update_result),
            _ => Err(Error::UnexpectedResponseType {
                expected: "UpdateServerResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn get_extension_providers(&mut self) -> Result<Vec<String>, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetExtensionProvidersRequest(
                GetExtensionProvidersRequest {},
            ))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetExtensionProvidersResponse(
                GetExtensionProvidersResponse { providers },
            )) => Ok(providers),
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetExtensionProvidersResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn search_extension(
        &mut self,
        provider: impl Into<String>,
        type_: ExtensionType,
        server_version: impl Into<String>,
        query: impl Into<String>,
        include_incompatible_versions: bool,
    ) -> Result<Vec<ExtensionInfo>, Error> {
        let response_payload = self
            .send_request(RequestPayload::SearchExtensionRequest(
                SearchExtensionRequest {
                    provider: provider.into(),
                    r#type: type_ as i32,
                    server_version: server_version.into(),
                    query: query.into(),
                    include_incompatible_versions,
                },
            ))
            .await?;

        match response_payload {
            Some(ResponsePayload::SearchExtensionResponse(SearchExtensionResponse {
                extensions,
            })) => Ok(extensions),
            _ => Err(Error::UnexpectedResponseType {
                expected: "SearchExtensionResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn get_extension_versions(
        &mut self,
        provider: impl Into<String>,
        type_: ExtensionType,
        server_version: impl Into<String>,
        extension_id: impl Into<String>,
        include_incompatible_versions: bool,
    ) -> Result<Vec<ExtensionVersionInfo>, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetExtensionVersionsRequest(
                GetExtensionVersionsRequest {
                    provider: provider.into(),
                    r#type: type_ as i32,
                    server_version: server_version.into(),
                    extension_id: extension_id.into(),
                    include_incompatible_versions,
                },
            ))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetExtensionVersionsResponse(GetExtensionVersionsResponse {
                versions,
            })) => Ok(versions),
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetExtensionVersionsResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_extension(
        &mut self,
        server_dir: impl Into<String>,
        provider: impl Into<String>,
        type_: ExtensionType,
        extension_id: impl Into<String>,
        extension_version_id: impl Into<String>,
        allow_incompatible_versions: bool,
        auto_update: bool,
    ) -> Result<AddExtensionResponse, Error> {
        let response_payload = self
            .send_request(RequestPayload::AddExtensionRequest(AddExtensionRequest {
                server_dir: server_dir.into(),
                provider: provider.into(),
                r#type: type_ as i32,
                extension_id: extension_id.into(),
                extension_version_id: extension_version_id.into(),
                allow_incompatible_versions,
                auto_update,
            }))
            .await?;

        match response_payload {
            Some(ResponsePayload::AddExtensionResponse(result)) => Ok(result),
            _ => Err(Error::UnexpectedResponseType {
                expected: "AddExtensionResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }

    pub async fn get_extension_id_by_url(
        &mut self,
        url: impl Into<String>,
    ) -> Result<GetExtensionIdByUrlResponse, Error> {
        let response_payload = self
            .send_request(RequestPayload::GetExtensionIdByUrlRequest(
                GetExtensionIdByUrlRequest { url: url.into() },
            ))
            .await?;

        match response_payload {
            Some(ResponsePayload::GetExtensionIdByUrlResponse(result)) => Ok(result),
            _ => Err(Error::UnexpectedResponseType {
                expected: "GetExtensionInfoByUrlResponse",
                actual: format!("{response_payload:?}"),
            }),
        }
    }
}

pub struct TerminalReader {
    stream_reader: OwnedReadHalf,
}
pub struct TerminalWriter {
    stream_writer: OwnedWriteHalf,
}

impl TerminalReader {
    pub async fn read(&mut self) -> Result<Option<TerminalOutput>, Error> {
        let length = match self.stream_reader.read_u32().await {
            Ok(len) => len,
            Err(_) => return Ok(None),
        };
        let mut buffer = vec![0u8; length as usize];
        self.stream_reader.read_exact(&mut buffer).await?;

        let output = TerminalOutput::decode(&buffer[..])?;
        Ok(Some(output))
    }
}

impl TerminalWriter {
    pub async fn write(&mut self, content: Vec<u8>) -> Result<(), Error> {
        let input = TerminalInput {
            input: Some(Input::Content(content)),
        };

        let bytes = input.encode_to_vec();
        self.stream_writer.write_u32(bytes.len() as u32).await?;
        self.stream_writer.write_all(&bytes).await?;

        Ok(())
    }

    pub async fn resize(&mut self, cols: u32, rows: u32) -> Result<(), Error> {
        let input = TerminalInput {
            input: Some(Input::Resize(TerminalSize { cols, rows })),
        };

        let bytes = input.encode_to_vec();
        self.stream_writer.write_u32(bytes.len() as u32).await?;
        self.stream_writer.write_all(&bytes).await?;

        Ok(())
    }
}
