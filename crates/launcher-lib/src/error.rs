#[derive(Debug, thiserror::Error)]
pub enum LibError {
    #[error("not file exists")]
    NoFileExists,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error("url parse: {0}")]
    UrlParse(String),

    #[error(transparent)]
    Zip(#[from] s_zip::SZipError),
}
