#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

#![allow(
    async_fn_in_trait,
)]

use std::{future::Future, path::Path, sync::Arc};

use error_stack::{report, ResultExt};
use futures::FutureExt;
use manager_model::{JsonRpcRequest, JsonRpcResponse, ManagerInstanceName, ManagerProtocolMode, ManagerProtocolVersion, ServerEvent};
use protocol::{ClientConnectionReadWrite, ConnectionUtilsRead, ConnectionUtilsWrite};
use tokio::net::TcpStream;
use tokio_rustls::{rustls::pki_types::{pem::PemObject, CertificateDer, ServerName}, TlsConnector};
use url::Url;

use error_stack::Result;

pub mod protocol;

pub use protocol::{ManagerClientWithRequestReceiver, RequestSenderCmds};
pub use tokio_rustls::rustls::RootCertStore;

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Write error")]
    Write,
    #[error("Read error")]
    Read,
    #[error("Flush error")]
    Flush,
    #[error("Parsing error")]
    Parse,
    #[error("Serializing error")]
    Serialize,
    #[error("Unsupported string length")]
    UnsupportedStringLength,
    #[error("Unsupported scheme")]
    UnsupportedScheme,
    #[error("Url host part is missing")]
    UrlHostMissing,
    #[error("Url host part is invalid")]
    UrlHostInvalid,
    #[error("Url port is missing")]
    UrlPortMissing,
    #[error("Connecting failed")]
    Connect,
    #[error("Root certificate is not configured")]
    RootCertificateIsNotConfigured,
    #[error("Root certificate loading error")]
    RootCertificateLoadingError,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Invalid API response")]
    InvalidResponse,
    #[error("Remote API request failed")]
    RemoteApiRequest,
    #[error("Local API request failed")]
    LocalApiRequest,

    #[error("Missing configuration")]
    MissingConfiguration,
}


#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub url: Url,
    /// Required for TLS connections
    pub root_certificate: Option<RootCertStore>,
    pub api_key: String,
}

pub struct ManagerClient {
    stream: Box<dyn ClientConnectionReadWrite>,
}


impl ManagerClient {
    pub fn load_root_certificate(root_certificate: impl AsRef<Path>) -> Result<RootCertStore, ClientError> {
        let certificate = CertificateDer::from_pem_file(root_certificate)
            .change_context(ClientError::RootCertificateLoadingError)?;

        let mut root_store = RootCertStore::empty();
        root_store.add( certificate)
            .change_context(ClientError::RootCertificateLoadingError)?;

        Ok(root_store)
    }

    pub async fn connect(config: ClientConfig) -> Result<Self, ClientError> {
        let host = config.url.host_str()
            .map(|v| v.to_string())
            .ok_or_else(|| report!(ClientError::UrlHostMissing))?;
        let port = config.url.port()
            .ok_or_else(|| report!(ClientError::UrlPortMissing))?;
        match config.url.scheme() {
            "tcp" => Self::connect_tcp(config, (host, port)).await,
            "tls" => Self::connect_tls(config, (host, port)).await,
            other => Err(report!(ClientError::UnsupportedScheme))
                .attach_printable(other.to_string()),
        }
    }

    async fn connect_tcp(config: ClientConfig, host_and_port: (String, u16)) -> Result<Self, ClientError> {
        let stream = TcpStream::connect(host_and_port)
            .await
            .change_context(ClientError::Connect)?;

        Self::init_connection(config, Box::new(stream)).await
    }

    async fn connect_tls(config: ClientConfig, host_and_port: (String, u16)) -> Result<Self, ClientError> {
        let domain = ServerName::try_from(host_and_port.0.clone())
            .change_context(ClientError::UrlHostInvalid)?;

        let Some(root_store) = config.root_certificate.clone() else {
            return Err(report!(ClientError::RootCertificateIsNotConfigured));
        };

        let tls_config = tokio_rustls::rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let stream = TcpStream::connect(host_and_port)
            .await
            .change_context(ClientError::Connect)?;
        let connector = TlsConnector::from(Arc::new(tls_config));
        let stream = connector.connect(domain, stream)
            .await
            .change_context(ClientError::Connect)?;

        Self::init_connection(config, Box::new(stream)).await
    }

    async fn init_connection(
        config: ClientConfig,
        mut stream: Box<dyn ClientConnectionReadWrite>
    ) -> Result<Self, ClientError> {
        stream.send_u8(ManagerProtocolVersion::V1 as u8)
            .await
            .change_context(ClientError::Write)?;
        stream.send_string_with_u32_len(config.api_key)
            .await
            .change_context(ClientError::Write)?;
        let result = stream.receive_u8()
            .await
            .change_context(ClientError::Read)?;
        if result != 1 {
            return Err(report!(ClientError::InvalidApiKey));
        }

        Ok(ManagerClient {
            stream,
        })
    }

    pub async fn send_request(
        mut self,
        request: JsonRpcRequest
    ) -> Result<JsonRpcResponse, ClientError> {
        self.send_request_internal(request).await
    }

    async fn send_request_internal(
        &mut self,
        request: JsonRpcRequest
    ) -> Result<JsonRpcResponse, ClientError> {
        self.stream.send_u8(ManagerProtocolMode::JsonRpc as u8)
            .await
            .change_context(ClientError::Write)?;
        self.stream.send_json_rpc_request(request)
            .await
            .change_context(ClientError::Write)?;
        self.stream.receive_json_rpc_response()
            .await
            .change_context(ClientError::Write)
    }

    pub async fn listen_events(
        mut self,
    ) -> Result<ServerEventListerner, ClientError> {
        self.stream.send_u8(ManagerProtocolMode::ListenServerEvents as u8)
            .await
            .change_context(ClientError::Write)?;
        Ok(ServerEventListerner { stream: self.stream })
    }

    pub fn request_to(
        self,
        request_receiver: ManagerInstanceName
    ) -> ManagerClientWithRequestReceiver {
        ManagerClientWithRequestReceiver {
            client: self,
            request_receiver,
        }
    }
}

pub struct ServerEventListerner {
    stream: Box<dyn ClientConnectionReadWrite>,
}

impl ServerEventListerner {
    pub async fn next_event(&mut self) -> Result<ServerEvent, ClientError> {
        self.stream.receive_server_event()
            .await
            .change_context(ClientError::Read)
    }
}

pub trait RequestSendingSupport {
    fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Box<dyn Future<Output = Result<JsonRpcResponse, ClientError>> + Send + '_>;
}

impl RequestSendingSupport for ManagerClient {
    fn send_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Box<dyn Future<Output = Result<JsonRpcResponse, ClientError>> + Send + '_> {
        Box::new(ManagerClient::send_request_internal(self, request).boxed())
    }
}
