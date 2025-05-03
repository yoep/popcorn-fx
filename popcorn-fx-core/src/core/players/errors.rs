use thiserror::Error;

/// The error type of the player manager..
pub type ManagerResult<T> = Result<T, ManagerError>;

#[derive(Debug, Clone, Error, PartialEq)]
pub enum RequestError {
    #[error("the play request url is missing")]
    UrlMissing,
    #[error("the play request title is missing")]
    TitleMissing,
    #[error("the play request media is missing")]
    MediaMissing,
}

/// The errors that can occur within the player manager.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ManagerError {
    #[error("player with id \"{0}\" has already been registered")]
    DuplicatePlayer(String),
}
