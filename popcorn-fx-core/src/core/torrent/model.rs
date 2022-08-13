/// The session state of a torrent service.
#[repr(C)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum SessionState {
    /// The torrent session is being created and not ready for use.
    CREATING = 0,
    /// The torrent session is being initialized and not ready for use.
    INITIALIZING = 1,
    /// The torrent session is running and ready for use.
    RUNNING = 2,
    /// The torrent session encountered an error and was unable to start correctly.
    ERROR = -1,
}