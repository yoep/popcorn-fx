use popcorn_fx_core::core::updater::{UpdateEvent, UpdateState};

use crate::popcorn::fx::ffi::VersionInfoC;

/// The C compatible callback for update events.
pub type UpdateCallbackC = extern "C" fn(UpdateEventC);

/// The C compatible update events.
#[repr(C)]
#[derive(Debug)]
pub enum UpdateEventC {
    /// Invoked when the state of the updater has changed
    StateChanged(UpdateStateC),
    /// Invoked when a new update is available
    UpdateAvailable(VersionInfoC),
}

impl From<UpdateEvent> for UpdateEventC {
    fn from(value: UpdateEvent) -> Self {
        match value {
            UpdateEvent::StateChanged(state) => UpdateEventC::StateChanged(UpdateStateC::from(state)),
            UpdateEvent::UpdateAvailable(version) => UpdateEventC::UpdateAvailable(VersionInfoC::from(&version)),
        }
    }
}

/// The C compatible update state
#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum UpdateStateC {
    CheckingForNewVersion = 0,
    UpdateAvailable = 1,
    NoUpdateAvailable = 2,
    Downloading = 3,
    /// Indicates that the download has finished.
    DownloadFinished = 4,
    Installing = 5,
    Error = 6,
}

impl From<UpdateState> for UpdateStateC{
    fn from(value: UpdateState) -> Self {
        match value {
            UpdateState::CheckingForNewVersion => UpdateStateC::CheckingForNewVersion,
            UpdateState::UpdateAvailable => UpdateStateC::UpdateAvailable,
            UpdateState::NoUpdateAvailable => UpdateStateC::NoUpdateAvailable,
            UpdateState::Downloading => UpdateStateC::Downloading,
            UpdateState::DownloadFinished(_) => UpdateStateC::DownloadFinished,
            UpdateState::Installing => UpdateStateC::Installing,
            UpdateState::Error => UpdateStateC::Error,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_update_event() {
        let event = UpdateEvent::StateChanged(UpdateState::UpdateAvailable);

        let result = UpdateEventC::from(event);

        match result {
            UpdateEventC::StateChanged(state) => assert_eq!(UpdateStateC::UpdateAvailable, state),
            _ => assert!(false, "expected UpdateEventC::StateChanged")
        }
    }

    #[test]
    fn test_from_update_state() {
        assert_eq!(UpdateStateC::CheckingForNewVersion, UpdateStateC::from(UpdateState::CheckingForNewVersion));
        assert_eq!(UpdateStateC::NoUpdateAvailable, UpdateStateC::from(UpdateState::NoUpdateAvailable));
        assert_eq!(UpdateStateC::UpdateAvailable, UpdateStateC::from(UpdateState::UpdateAvailable));
        assert_eq!(UpdateStateC::Downloading, UpdateStateC::from(UpdateState::Downloading));
        assert_eq!(UpdateStateC::DownloadFinished, UpdateStateC::from(UpdateState::DownloadFinished(String::new())));
    }
}