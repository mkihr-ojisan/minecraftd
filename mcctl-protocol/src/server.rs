use std::path::Path;

use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use crate::{
    request::Payload as RequestPayload, response::Payload as ResponsePayload,
    terminal_input::Input, *,
};

pub trait RequestHandler<E, R, W>
where
    E: Send + 'static,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
{
    fn get_server_implementations() -> impl Future<Output = Result<Vec<String>, E>> + Send;
    fn get_versions(
        server_implementation: &str,
    ) -> impl Future<Output = Result<Vec<Version>, E>> + Send;
    fn get_builds(
        server_implementation: &str,
        version: &str,
    ) -> impl Future<Output = Result<Vec<Build>, E>> + Send;
    fn create_server(
        name: &str,
        server_dir: &Path,
        server_implementation: &str,
        version: &str,
        build: &str,
        connection: ConnectionType,
        hostname: Option<&str>,
    ) -> impl Future<Output = Result<(), E>> + Send;
    fn start_server(server_dir: &Path) -> impl Future<Output = Result<(), E>> + Send;
    fn stop_server(server_dir: &Path) -> impl Future<Output = Result<(), E>> + Send;
    fn kill_server(server_dir: &Path) -> impl Future<Output = Result<(), E>> + Send;
    fn attach_terminal(server_dir: &Path) -> impl Future<Output = Result<(R, W), E>> + Send;
    fn get_running_servers() -> impl Future<Output = Result<Vec<RunningServer>, E>> + Send;
    fn wait_ready(server_dir: &Path) -> impl Future<Output = Result<(), E>> + Send;
    fn restart_server(server_dir: &Path) -> impl Future<Output = Result<(), E>> + Send;
    fn update_server(
        server_dir: &Path,
        update_type: UpdateType,
    ) -> impl Future<Output = Result<UpdateServerResponse, E>> + Send;
    fn get_extension_providers() -> impl Future<Output = Result<Vec<String>, E>> + Send;
    fn search_extension(
        provider: &str,
        type_: ExtensionType,
        server_version: &str,
        query: &str,
        include_incompatible_versions: bool,
    ) -> impl Future<Output = Result<Vec<ExtensionInfo>, E>> + Send;
    fn get_extension_versions(
        provider: &str,
        type_: ExtensionType,
        server_version: &str,
        extension_id: &str,
        include_incompatible_versions: bool,
    ) -> impl Future<Output = Result<Vec<ExtensionVersionInfo>, E>> + Send;
    fn add_extension(
        server_dir: &Path,
        provider: &str,
        type_: ExtensionType,
        extension_id: &str,
        extension_version_id: &str,
        allow_incompatible_versions: bool,
    ) -> impl Future<Output = Result<AddExtensionResponse, E>> + Send;
}

pub trait TerminalReader<E>: Send + 'static
where
    E: Send + 'static,
{
    fn read(&mut self) -> impl Future<Output = Result<Option<TerminalOutput>, E>> + Send;
}

pub trait TerminalWriter<E>: Send + 'static
where
    E: Send + 'static,
{
    fn write(&mut self, data: &[u8]) -> impl Future<Output = Result<(), E>> + Send;
    fn resize(&mut self, cols: u16, rows: u16) -> impl Future<Output = Result<(), E>> + Send;
}

pub async fn listen<E, R, W, H>(
    shutdown_signal: impl Future,
    error_to_string: fn(&E) -> String,
) -> Result<(), Error>
where
    E: Send + 'static,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
    H: RequestHandler<E, R, W>,
{
    let socket_path = socket_path()?;

    let _ = std::fs::remove_file(&socket_path);

    let listener = UnixListener::bind(&socket_path)?;

    info!("Listening on socket {:?}", socket_path);

    let finalize = || {
        let _ = std::fs::remove_file(&socket_path);
    };

    let listen = async move {
        loop {
            let (stream, _) = listener.accept().await?;
            debug!("Accepted new connection");

            tokio::spawn(async move {
                if let Err(err) = handle_client::<E, R, W, H>(stream, error_to_string).await {
                    error!("Error handling client: {:?}", err);
                }
            });
        }
        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    };

    tokio::select! {
        _ = shutdown_signal => {}
        result = listen => {
            result?;
        }
    }

    finalize();
    Ok(())
}

async fn handle_client<E, R, W, H>(
    mut stream: UnixStream,
    error_to_string: fn(&E) -> String,
) -> Result<(), Error>
where
    E: Send + 'static,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
    H: RequestHandler<E, R, W>,
{
    let mut buf = vec![0u8; 1024];

    loop {
        let length = match stream.read_u32().await {
            Ok(len) => Ok(len),
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("Client disconnected");
                break;
            }
            Err(e) => Err(e),
        }?;

        buf.resize(length as usize, 0);
        stream.read_exact(&mut buf).await?;

        let request: Request = Request::decode(&buf[..])?;

        debug!("Received request: {:?}", request);

        let response_payload = match request.payload {
            Some(payload) => match handle_request::<E, R, W, H>(payload).await {
                Ok(payload) => payload,
                Err(e) => {
                    warn!("Error handling request: {}", e.to_string(error_to_string));
                    HandleRequestResult::Response(Some(ResponsePayload::ErrorResponse(
                        ErrorResponse {
                            message: e.to_string(error_to_string),
                        },
                    )))
                }
            },
            None => {
                HandleRequestResult::Response(Some(ResponsePayload::ErrorResponse(ErrorResponse {
                    message: "Received request with no payload".to_string(),
                })))
            }
        };

        match response_payload {
            HandleRequestResult::Response(payload) => {
                let response = Response { payload };

                let mut response_buf = Vec::new();
                response.encode(&mut response_buf).unwrap();
                let response_length = response_buf.len() as u32;

                stream.write_u32(response_length).await?;
                stream.write_all(&response_buf).await?
            }
            HandleRequestResult::AttachTerminal(terminal_reader, terminal_writer, _) => {
                let response = Response { payload: None };
                let mut response_buf = Vec::new();
                response.encode(&mut response_buf).unwrap();
                let response_length = response_buf.len() as u32;

                stream.write_u32(response_length).await?;
                stream.write_all(&response_buf).await?;

                handle_terminal_connection::<E, R, W>(
                    stream,
                    terminal_reader,
                    terminal_writer,
                    error_to_string,
                )
                .await?;
                return Ok(());
            }
        };
    }

    Ok(())
}

enum HandleRequestResult<E, R, W>
where
    E: Send + 'static,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
{
    Response(Option<ResponsePayload>),
    AttachTerminal(R, W, std::marker::PhantomData<E>),
}

#[derive(Debug)]
enum HandleRequestError<E: Send + 'static> {
    Error(Error),
    HandlerError(E),
}
impl<E: Send + 'static> HandleRequestError<E> {
    fn to_string(&self, error_to_string: fn(&E) -> String) -> String {
        match self {
            HandleRequestError::Error(e) => e.to_string(),
            HandleRequestError::HandlerError(e) => error_to_string(e),
        }
    }
}
impl<E: Send + 'static> From<E> for HandleRequestError<E> {
    fn from(value: E) -> Self {
        HandleRequestError::HandlerError(value)
    }
}

async fn handle_request<E, R, W, H>(
    request_payload: RequestPayload,
) -> Result<HandleRequestResult<E, R, W>, HandleRequestError<E>>
where
    E: Send + 'static,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
    H: RequestHandler<E, R, W>,
{
    match request_payload {
        RequestPayload::GetServerImplementationsRequest(_) => {
            let response_payload = ResponsePayload::GetServerImplementationsResponse(
                GetServerImplementationsResponse {
                    implementations: H::get_server_implementations().await?,
                },
            );

            Ok(HandleRequestResult::Response(Some(response_payload)))
        }
        RequestPayload::GetVersionsRequest(req) => {
            let versions = H::get_versions(&req.server_implementation).await?;

            let response_payload =
                ResponsePayload::GetVersionsResponse(GetVersionsResponse { versions });

            Ok(HandleRequestResult::Response(Some(response_payload)))
        }
        RequestPayload::GetBuildsRequest(req) => {
            let builds = H::get_builds(&req.server_implementation, &req.version).await?;

            let response_payload = ResponsePayload::GetBuildsResponse(GetBuildsResponse { builds });

            Ok(HandleRequestResult::Response(Some(response_payload)))
        }
        RequestPayload::CreateServerRequest(req) => {
            H::create_server(
                &req.name,
                Path::new(&req.server_dir),
                &req.server_implementation,
                &req.version,
                &req.build,
                ConnectionType::try_from(req.connection)
                    .map_err(|_| HandleRequestError::Error(Error::InvalidConnectionType))?,
                req.hostname.as_deref(),
            )
            .await?;

            Ok(HandleRequestResult::Response(None))
        }
        RequestPayload::StartServerRequest(req) => {
            H::start_server(Path::new(&req.server_dir)).await?;

            Ok(HandleRequestResult::Response(None))
        }
        RequestPayload::StopServerRequest(req) => {
            H::stop_server(Path::new(&req.server_dir)).await?;

            Ok(HandleRequestResult::Response(None))
        }
        RequestPayload::KillServerRequest(req) => {
            H::kill_server(Path::new(&req.server_dir)).await?;

            Ok(HandleRequestResult::Response(None))
        }
        RequestPayload::AttachTerminalRequest(req) => {
            let (terminal_reader, terminal_writer) =
                H::attach_terminal(Path::new(&req.server_dir)).await?;

            Ok(HandleRequestResult::AttachTerminal(
                terminal_reader,
                terminal_writer,
                std::marker::PhantomData,
            ))
        }
        RequestPayload::GetRunningServersRequest(_) => {
            let response_payload =
                ResponsePayload::GetRunningServersResponse(GetRunningServersResponse {
                    servers: H::get_running_servers().await?,
                });

            Ok(HandleRequestResult::Response(Some(response_payload)))
        }
        RequestPayload::WaitServerReadyRequest(req) => {
            H::wait_ready(Path::new(&req.server_dir)).await?;

            Ok(HandleRequestResult::Response(None))
        }
        RequestPayload::RestartServerRequest(req) => {
            H::restart_server(Path::new(&req.server_dir)).await?;

            Ok(HandleRequestResult::Response(None))
        }
        RequestPayload::UpdateServerRequest(req) => {
            let update_result = H::update_server(
                Path::new(&req.server_dir),
                UpdateType::try_from(req.update_type)
                    .map_err(|_| HandleRequestError::Error(Error::InvalidUpdateType))?,
            )
            .await?;

            Ok(HandleRequestResult::Response(Some(
                ResponsePayload::UpdateServerResponse(update_result),
            )))
        }
        RequestPayload::GetExtensionProvidersRequest(_) => {
            let providers = H::get_extension_providers().await?;

            Ok(HandleRequestResult::Response(Some(
                ResponsePayload::GetExtensionProvidersResponse(GetExtensionProvidersResponse {
                    providers,
                }),
            )))
        }
        RequestPayload::SearchExtensionRequest(req) => {
            let extensions = H::search_extension(
                &req.provider,
                ExtensionType::try_from(req.r#type)
                    .map_err(|_| HandleRequestError::Error(Error::InvalidExtensionType))?,
                &req.server_version,
                &req.query,
                req.include_incompatible_versions,
            )
            .await?;

            Ok(HandleRequestResult::Response(Some(
                ResponsePayload::SearchExtensionResponse(SearchExtensionResponse { extensions }),
            )))
        }
        RequestPayload::GetExtensionVersionsRequest(req) => {
            let versions = H::get_extension_versions(
                &req.provider,
                ExtensionType::try_from(req.r#type)
                    .map_err(|_| HandleRequestError::Error(Error::InvalidExtensionType))?,
                &req.server_version,
                &req.extension_id,
                req.include_incompatible_versions,
            )
            .await?;

            Ok(HandleRequestResult::Response(Some(
                ResponsePayload::GetExtensionVersionsResponse(GetExtensionVersionsResponse {
                    versions,
                }),
            )))
        }
        RequestPayload::AddExtensionRequest(req) => {
            let result = H::add_extension(
                Path::new(&req.server_dir),
                &req.provider,
                ExtensionType::try_from(req.r#type)
                    .map_err(|_| HandleRequestError::Error(Error::InvalidExtensionType))?,
                &req.extension_id,
                &req.extension_version_id,
                req.allow_incompatible_versions,
            )
            .await?;

            Ok(HandleRequestResult::Response(Some(
                ResponsePayload::AddExtensionResponse(result),
            )))
        }
    }
}

async fn handle_terminal_connection<E, R, W>(
    stream: UnixStream,
    mut terminal_reader: R,
    mut terminal_writer: W,
    error_to_string: fn(&E) -> String,
) -> Result<(), Error>
where
    E: Send + 'static,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
{
    let (mut stream_reader, mut stream_writer) = stream.into_split();

    let input_task = tokio::spawn(async move {
        let result: Result<(), HandleRequestError<E>> = async {
            let mut buffer = Vec::new();

            loop {
                let length = match stream_reader.read_u32().await {
                    Ok(len) => Ok(len),
                    Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                        debug!("Terminal client disconnected");
                        return Ok(());
                    }
                    Err(e) => Err(e),
                }
                .map_err(|e| HandleRequestError::Error(e.into()))?;
                buffer.resize(length as usize, 0);
                stream_reader
                    .read_exact(&mut buffer)
                    .await
                    .map_err(|e| HandleRequestError::Error(e.into()))?;

                let input = TerminalInput::decode(&buffer[..])
                    .map_err(|e| HandleRequestError::Error(e.into()))?;

                match input
                    .input
                    .ok_or(HandleRequestError::Error(Error::NoPayload))?
                {
                    Input::Content(content) => {
                        terminal_writer.write(&content).await?;
                    }
                    Input::Resize(size) => {
                        terminal_writer
                            .resize(size.cols as u16, size.rows as u16)
                            .await?;
                    }
                }
            }
        }
        .await;

        if let Err(e) = result {
            error!(
                "Error in terminal input task: {}",
                e.to_string(error_to_string)
            );
        }
    });

    let output_task = tokio::spawn(async move {
        let result: Result<(), HandleRequestError<E>> = async {
            loop {
                let Some(output) = terminal_reader.read().await? else {
                    break Ok(());
                };

                trace!(
                    "Terminal output: {:?}",
                    String::from_utf8_lossy(&output.content)
                );

                let mut output_buf = Vec::new();
                output
                    .encode(&mut output_buf)
                    .map_err(|e| HandleRequestError::Error(e.into()))?;

                let output_length = output_buf.len() as u32;

                stream_writer
                    .write_u32(output_length)
                    .await
                    .map_err(|e| HandleRequestError::Error(e.into()))?;
                stream_writer
                    .write_all(&output_buf)
                    .await
                    .map_err(|e| HandleRequestError::Error(e.into()))?;
            }
        }
        .await;

        if let Err(e) = result {
            error!(
                "Error in terminal output task: {}",
                e.to_string(error_to_string)
            );
        }
    });

    tokio::try_join!(input_task, output_task).unwrap();

    Ok(())
}
