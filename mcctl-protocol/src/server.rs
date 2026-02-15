use std::{fmt::Display, path::Path};

use prost::Message;
use thiserror::Error;
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
    E: Display + Send,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
{
    fn get_server_implementations() -> impl Future<Output = Result<Vec<String>, E>> + Send;
    fn get_versions(
        server_implementation: &str,
    ) -> impl Future<Output = Result<Vec<String>, E>> + Send;
    fn get_builds(
        server_implementation: &str,
        version: &str,
    ) -> impl Future<Output = Result<Vec<String>, E>> + Send;
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
}

pub trait TerminalReader<E>: Send + 'static
where
    E: Display + Send,
{
    fn read(&mut self) -> impl Future<Output = Result<Option<TerminalOutput>, E>> + Send;
}

pub trait TerminalWriter<E>: Send + 'static
where
    E: Display + Send,
{
    fn write(&mut self, data: &[u8]) -> impl Future<Output = Result<(), E>> + Send;
    fn resize(&mut self, cols: u16, rows: u16) -> impl Future<Output = Result<(), E>> + Send;
}

pub async fn listen<E, R, W, H>(shutdown_signal: impl Future) -> Result<(), Error>
where
    E: Display + Send,
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

    let listen = async {
        loop {
            let (stream, _) = listener.accept().await?;
            debug!("Accepted new connection");

            tokio::spawn(async {
                if let Err(err) = handle_client::<E, R, W, H>(stream).await {
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

async fn handle_client<E, R, W, H>(mut stream: UnixStream) -> Result<(), Error>
where
    E: Display + Send,
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
                    warn!("Error handling request: {}", e);
                    HandleRequestResult::Response(Some(ResponsePayload::ErrorResponse(
                        ErrorResponse {
                            message: format!("{e}"),
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

                handle_terminal_connection::<E, R, W>(stream, terminal_reader, terminal_writer)
                    .await?;
                return Ok(());
            }
        };
    }

    Ok(())
}

enum HandleRequestResult<E, R, W>
where
    E: Display + Send,
    R: TerminalReader<E>,
    W: TerminalWriter<E>,
{
    Response(Option<ResponsePayload>),
    AttachTerminal(R, W, std::marker::PhantomData<E>),
}

#[derive(Debug, Error)]
enum HandleRequestError<E: Display + Send> {
    #[error("{0}")]
    Error(Error),
    #[error("{0}")]
    HandlerError(#[from] E),
}

async fn handle_request<E, R, W, H>(
    request_payload: RequestPayload,
) -> Result<HandleRequestResult<E, R, W>, HandleRequestError<E>>
where
    E: Display + Send,
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
    }
}

async fn handle_terminal_connection<E, R, W>(
    stream: UnixStream,
    mut terminal_reader: R,
    mut terminal_writer: W,
) -> Result<(), Error>
where
    E: Display + Send,
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
            error!("Error in terminal input task: {e}");
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
            error!("Error in terminal output task: {e}");
        }
    });

    tokio::try_join!(input_task, output_task).unwrap();

    Ok(())
}
