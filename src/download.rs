use {
    crate::{
        Jou,
        Trn,
        Wtb,
    },
    hyper::{
        body::{
            to_bytes,
            Bytes,
        },
        client::{
            Client,
            HttpConnector,
        },
        Error as HyperError,
        StatusCode,
    },
    hyper_rustls::{
        HttpsConnector,
        HttpsConnectorBuilder,
    },
    std::{
        error::Error,
        fmt::{
            Debug,
            Display,
            Formatter,
            Result as FmtResult,
        },
    },
};

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

    /// Downloads a jou file.
    pub async fn jou(&self) -> Result<Bytes, DownloadError> {
        let mut name = Jou::file_stem().to_uppercase();
        name.push_str(".JOU");
        self.download(&name).await
    }

    /// Downloads a trn file.
    pub async fn trn(&self) -> Result<Bytes, DownloadError> {
        let mut name = Trn::file_stem().to_uppercase();
        name.push_str(".TRN");
        self.download(&name).await
    }

    /// Downloads a wtb file.
    pub async fn wtb(&self, year: u16) -> Result<Bytes, DownloadError> {
        let mut name = Wtb::file_stem(year);
        name.make_ascii_uppercase();
        name.push_str(".wtb");
        self.download(&name).await
    }

    async fn download(&self, name: &str) -> Result<Bytes, DownloadError> {
        let response = self
            .client
            .get(
                format!("https://www.ffothello.org/wthor/base/{name}")
                    .parse()
                    .unwrap(),
            )
            .await?;

        match response.status() {
            StatusCode::OK => Result::Ok(to_bytes(response.into_body()).await?),
            _ => Result::Err(DownloadError::StatusCode(response.status())),
        }
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
    StatusCode(StatusCode),
    Hyper(HyperError),
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::StatusCode(code) => Display::fmt(&code, f),
            Self::Hyper(error) => Display::fmt(&error, f),
        }
    }
}

impl Error for DownloadError {}

impl From<HyperError> for DownloadError {
    fn from(error: HyperError) -> Self {
        Self::Hyper(error)
    }
}
