use crate::ipc::proto::update;
use crate::ipc::proto::update::update::error;
use crate::ipc::proto::update::update_event;
use popcorn_fx_core::core::updater::{
    DownloadProgress, PatchInfo, UpdateError, UpdateEvent, UpdateState, VersionInfo,
};
use protobuf::MessageField;

impl From<&UpdateState> for update::update::State {
    fn from(value: &UpdateState) -> Self {
        match value {
            UpdateState::CheckingForNewVersion => Self::CHECKING_FOR_NEW_VERSION,
            UpdateState::UpdateAvailable => Self::UPDATE_AVAILABLE,
            UpdateState::NoUpdateAvailable => Self::NO_UPDATE_AVAILABLE,
            UpdateState::Downloading => Self::DOWNLOADING,
            UpdateState::DownloadFinished => Self::DOWNLOAD_FINISHED,
            UpdateState::Installing => Self::INSTALLING,
            UpdateState::InstallationFinished => Self::INSTALLING,
            UpdateState::Error => Self::ERROR,
        }
    }
}

impl From<&VersionInfo> for update::update::VersionInfo {
    fn from(value: &VersionInfo) -> Self {
        Self {
            application: MessageField::some(update::update::PatchInfo::from(&value.application)),
            runtime: MessageField::some(update::update::PatchInfo::from(&value.runtime)),
            special_fields: Default::default(),
        }
    }
}

impl From<&PatchInfo> for update::update::PatchInfo {
    fn from(value: &PatchInfo) -> Self {
        Self {
            version: value.version.clone(),
            platforms: value.platforms.clone(),
            special_fields: Default::default(),
        }
    }
}

impl From<&DownloadProgress> for update::update::DownloadProgress {
    fn from(value: &DownloadProgress) -> Self {
        Self {
            total_size: value.total_size,
            downloaded: value.downloaded,
            special_fields: Default::default(),
        }
    }
}

impl From<&UpdateEvent> for update::UpdateEvent {
    fn from(value: &UpdateEvent) -> Self {
        let mut event = Self::new();

        match value {
            UpdateEvent::StateChanged(state) => {
                event.event = update_event::Event::STATE_CHANGED.into();
                event.state_changed = MessageField::some(update_event::StateChanged {
                    new_state: update::update::State::from(state).into(),
                    special_fields: Default::default(),
                });
            }
            UpdateEvent::UpdateAvailable(info) => {
                event.event = update_event::Event::UPDATE_AVAILABLE.into();
                event.update_available = MessageField::some(update_event::UpdateAvailable {
                    version_info: MessageField::some(update::update::VersionInfo::from(info)),
                    special_fields: Default::default(),
                });
            }
            UpdateEvent::DownloadProgress(progress) => {
                event.event = update_event::Event::DOWNLOAD_PROGRESS.into();
                event.download_progress = MessageField::some(update_event::DownloadProgress {
                    progress: MessageField::some(update::update::DownloadProgress::from(progress)),
                    special_fields: Default::default(),
                });
            }
            UpdateEvent::InstallationProgress(_) => {
                event.event = update_event::Event::INSTALLATION_PROGRESS.into();
            }
        }

        event
    }
}

impl From<&UpdateError> for update::update::Error {
    fn from(value: &UpdateError) -> Self {
        let mut err = Self::new();

        match value {
            UpdateError::InvalidUpdateChannel(channel) => {
                err.type_ = error::Type::INVALID_UPDATE_CHANNEL.into();
                err.invalid_update_channel = MessageField::some(error::InvalidUpdateChannel {
                    channel: channel.clone(),
                    special_fields: Default::default(),
                });
            }
            UpdateError::InvalidApplicationVersion(version, reason) => {
                err.type_ = error::Type::INVALID_APPLICATION_VERSION.into();
                err.invalid_application_version =
                    MessageField::some(error::InvalidApplicationVersion {
                        version_value: version.clone(),
                        reason: reason.clone(),
                        special_fields: Default::default(),
                    });
            }
            UpdateError::InvalidRuntimeVersion(version, reason) => {
                err.type_ = error::Type::INVALID_RUNTIME_VERSION.into();
                err.invalid_runtime_version = MessageField::some(error::InvalidRuntimeVersion {
                    version_value: version.clone(),
                    reason: reason.clone(),
                    special_fields: Default::default(),
                });
            }
            UpdateError::UnknownVersion => {
                err.type_ = error::Type::UNKNOWN_VERSION.into();
            }
            UpdateError::Response(response) => {
                err.type_ = error::Type::RESPONSE.into();
                err.invalid_response = MessageField::some(error::InvalidResponse {
                    reason: response.clone(),
                    special_fields: Default::default(),
                });
            }
            UpdateError::InvalidDownloadUrl(url) => {
                err.type_ = error::Type::INVALID_DOWNLOAD_URL.into();
                err.invalid_download_url = MessageField::some(error::InvalidDownloadUrl {
                    url: url.clone(),
                    special_fields: Default::default(),
                });
            }
            UpdateError::PlatformUpdateUnavailable => {}
            UpdateError::DownloadFailed(_, _) => {}
            UpdateError::IO(_) => {}
            UpdateError::UpdateNotAvailable(_) => {}
            UpdateError::ExtractionFailed(_) => {}
            UpdateError::ArchiveLocationAlreadyExists => {}
        }

        err
    }
}
