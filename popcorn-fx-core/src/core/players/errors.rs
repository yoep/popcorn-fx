use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum RequestError {
    #[error("the play request url is missing")]
    UrlMissing,
    #[error("the play request title is missing")]
    TitleMissing,
    #[error("the play request media is missing")]
    MediaMissing,
}
