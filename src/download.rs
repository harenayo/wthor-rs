use {
    crate::{
        Jou,
        ReadError,
        Trn,
        Wtb,
    },
    hyper::{
        body::{
            aggregate,
            Buf as _,
        },
        client::{
            Client,
            HttpConnector,
        },
        http::uri::InvalidUri as UriError,
        Error as HyperError,
        StatusCode,
        Uri,
    },
    hyper_rustls::{
        HttpsConnector,
        HttpsConnectorBuilder,
    },
    std::{
        error::Error,
        fmt::{
            Display,
            Formatter,
            Result as FmtResult,
        },
        io::Read,
    },
};

macro_rules! uri {
    ($name:literal) => {
        concat!("https://www.ffothello.org/wthor/base/", $name)
    };
}

/// A database file downloader.
#[derive(Clone, Debug)]
pub struct Downloader {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl Downloader {
    /// Creates a new downloader.
    pub fn new() -> Self {
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);

        Self {
            client: Client::builder().build(
                HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http2()
                    .wrap_connector(connector),
            ),
        }
    }

    async fn download(&self, uri: Uri) -> Result<impl Read, DownloadError> {
        let response = self.client.get(uri).await?;

        match response.status() {
            StatusCode::OK => Result::Ok(aggregate(response).await?.reader()),
            _ => Result::Err(DownloadError::StatusCode(response.status())),
        }
    }
}

impl Jou {
    /// Downloads a file.
    pub async fn download(downloader: &Downloader) -> Result<Self, DownloadError> {
        Result::Ok(Self::read(
            downloader
                .download(Uri::from_static(uri!("WTHOR.JOU")))
                .await?,
        )?)
    }
}

impl Trn {
    /// Downloads a file.
    pub async fn download(downloader: &Downloader) -> Result<Self, DownloadError> {
        Result::Ok(Self::read(
            downloader
                .download(Uri::from_static(uri!("WTHOR.TRN")))
                .await?,
        )?)
    }
}

impl Wtb {
    /// Downloads a file.
    pub async fn download(downloader: &Downloader, year: u16) -> Result<Self, DownloadError> {
        Result::Ok(Self::read(
            downloader
                .download(format!(uri!("WTH_{}.wtb"), year).parse()?)
                .await?,
        )?)
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

/// An error while downloading.
#[derive(Debug)]
pub enum DownloadError {
    /// See [`InvalidUri`](UriError).
    Uri(UriError),
    /// See [`StatusCode`].
    StatusCode(StatusCode),
    /// See [`Error`](HyperError).
    Hyper(HyperError),
    /// See [`ReadError`].
    Read(ReadError),
}

impl Display for DownloadError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Uri(error) => error.fmt(formatter),
            Self::StatusCode(code) => code.fmt(formatter),
            Self::Hyper(error) => error.fmt(formatter),
            Self::Read(error) => error.fmt(formatter),
        }
    }
}

impl Error for DownloadError {}

impl From<UriError> for DownloadError {
    fn from(error: UriError) -> Self {
        Self::Uri(error)
    }
}

impl From<HyperError> for DownloadError {
    fn from(error: HyperError) -> Self {
        Self::Hyper(error)
    }
}

impl From<ReadError> for DownloadError {
    fn from(error: ReadError) -> Self {
        Self::Read(error)
    }
}
