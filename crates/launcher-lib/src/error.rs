#[derive(Debug, thiserror::Error)]
pub enum LibError {
    #[error("not file exists")]
    NoFileExists,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}
