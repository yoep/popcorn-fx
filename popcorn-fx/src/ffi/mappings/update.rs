use popcorn_fx_core::core::updater::{
    DownloadProgress, InstallationProgress, UpdateEvent, UpdateState,
};

use crate::ffi::VersionInfoC;

/// The C compatible callback for update events.
pub type UpdateCallbackC = extern "C" fn(UpdateEventC);

/// The C compatible representation of the update events.
///
/// This enum maps to the `UpdateEvent` enum but with C-compatible data types.
///
/// # Fields
///
/// * `StateChanged(state)` - Invoked when the state of the updater has changed
/// * `UpdateAvailable(version)` - Invoked when a new update is available
/// * `DownloadProgress(progress)` - Invoked when the update download progresses
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum UpdateEventC {
    StateChanged(UpdateStateC),
    UpdateAvailable(VersionInfoC),
    DownloadProgress(DownloadProgressC),
    InstallationProgress(InstallationProgressC),
}

impl From<UpdateEvent> for UpdateEventC {
    fn from(value: UpdateEvent) -> Self {
        match value {
            UpdateEvent::StateChanged(state) => {
                UpdateEventC::StateChanged(UpdateStateC::from(state))
            }
            UpdateEvent::UpdateAvailable(version) => {
                UpdateEventC::UpdateAvailable(VersionInfoC::from(&version))
            }
            UpdateEvent::DownloadProgress(progress) => {
                UpdateEventC::DownloadProgress(DownloadProgressC::from(progress))
            }
            UpdateEvent::InstallationProgress(progress) => {
                UpdateEventC::InstallationProgress(InstallationProgressC::from(progress))
            }
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
    InstallationFinished = 6,
    Error = 7,
}

impl From<UpdateState> for UpdateStateC {
    fn from(value: UpdateState) -> Self {
        match value {
            UpdateState::CheckingForNewVersion => UpdateStateC::CheckingForNewVersion,
            UpdateState::UpdateAvailable => UpdateStateC::UpdateAvailable,
            UpdateState::NoUpdateAvailable => UpdateStateC::NoUpdateAvailable,
            UpdateState::Downloading => UpdateStateC::Downloading,
            UpdateState::DownloadFinished => UpdateStateC::DownloadFinished,
            UpdateState::Installing => UpdateStateC::Installing,
            UpdateState::InstallationFinished => UpdateStateC::InstallationFinished,
            UpdateState::Error => UpdateStateC::Error,
        }
    }
}

/// The C-compatible representation of the [DownloadProgress] struct.
///
/// This struct is used to provide C code access to the download progress of an update event.
///
/// # Fields
///
/// * `total_size` - The total size of the update download in bytes.
/// * `downloaded` - The total number of bytes downloaded so far.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct DownloadProgressC {
    pub total_size: u64,
    pub downloaded: u64,
}

impl From<DownloadProgress> for DownloadProgressC {
    fn from(value: DownloadProgress) -> Self {
        Self {
            total_size: value.total_size,
            downloaded: value.downloaded,
        }
    }
}

/// The C-compatible representation of the [InstallationProgress] struct.
///
/// This struct is used to provide C code access to the installation progress of an update event.
///
/// # Fields
///
/// * `task` - The current task being executed during the installation process.
/// * `total_tasks` - The total number of tasks that need to be executed during the installation process.
/// * `task_progress` - The current progress of the current task, represented as a fraction between 0.0 and 1.0.
#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct InstallationProgressC {
    pub task: u16,
    pub total_tasks: u16,
}

impl From<InstallationProgress> for InstallationProgressC {
    fn from(value: InstallationProgress) -> Self {
        Self {
            task: value.task,
            total_tasks: value.total_tasks,
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
            _ => assert!(false, "expected UpdateEventC::StateChanged"),
        }
    }

    #[test]
    fn test_from_update_event_download_progress() {
        let progress = DownloadProgress {
            total_size: 1024,
            downloaded: 512,
        };
        let event = UpdateEvent::DownloadProgress(progress.clone());
        let expected = UpdateEventC::DownloadProgress(DownloadProgressC::from(progress));

        let actual = UpdateEventC::from(event);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_from_update_state() {
        assert_eq!(
            UpdateStateC::CheckingForNewVersion,
            UpdateStateC::from(UpdateState::CheckingForNewVersion)
        );
        assert_eq!(
            UpdateStateC::NoUpdateAvailable,
            UpdateStateC::from(UpdateState::NoUpdateAvailable)
        );
        assert_eq!(
            UpdateStateC::UpdateAvailable,
            UpdateStateC::from(UpdateState::UpdateAvailable)
        );
        assert_eq!(
            UpdateStateC::Downloading,
            UpdateStateC::from(UpdateState::Downloading)
        );
        assert_eq!(
            UpdateStateC::DownloadFinished,
            UpdateStateC::from(UpdateState::DownloadFinished)
        );
        assert_eq!(
            UpdateStateC::Installing,
            UpdateStateC::from(UpdateState::Installing)
        );
        assert_eq!(
            UpdateStateC::InstallationFinished,
            UpdateStateC::from(UpdateState::InstallationFinished)
        );
    }

    #[test]
    fn test_from_download_progress() {
        let progress = DownloadProgress {
            total_size: 1024,
            downloaded: 512,
        };

        let progress_c = DownloadProgressC::from(progress);

        assert_eq!(progress_c.total_size, 1024);
        assert_eq!(progress_c.downloaded, 512);
    }

    #[test]
    fn test_installation_progress_conversions() {
        let progress = InstallationProgress {
            task: 3,
            total_tasks: 10,
        };

        let c_progress = InstallationProgressC::from(progress.clone());

        assert_eq!(c_progress.task, progress.task);
        assert_eq!(c_progress.total_tasks, progress.total_tasks);
    }
}
