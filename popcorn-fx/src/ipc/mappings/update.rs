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
            UpdateState::InstallationFinished => Self::INSTALLATION_FINISHED,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_state_from() {
        assert_eq!(
            update::update::State::CHECKING_FOR_NEW_VERSION,
            update::update::State::from(&UpdateState::CheckingForNewVersion)
        );
        assert_eq!(
            update::update::State::UPDATE_AVAILABLE,
            update::update::State::from(&UpdateState::UpdateAvailable)
        );
        assert_eq!(
            update::update::State::NO_UPDATE_AVAILABLE,
            update::update::State::from(&UpdateState::NoUpdateAvailable)
        );
        assert_eq!(
            update::update::State::DOWNLOADING,
            update::update::State::from(&UpdateState::Downloading)
        );
        assert_eq!(
            update::update::State::DOWNLOAD_FINISHED,
            update::update::State::from(&UpdateState::DownloadFinished)
        );
        assert_eq!(
            update::update::State::INSTALLING,
            update::update::State::from(&UpdateState::Installing)
        );
        assert_eq!(
            update::update::State::INSTALLATION_FINISHED,
            update::update::State::from(&UpdateState::InstallationFinished)
        );
        assert_eq!(
            update::update::State::ERROR,
            update::update::State::from(&UpdateState::Error)
        );
    }

    #[test]
    fn test_update_event_from_state_changed() {
        let event = UpdateEvent::StateChanged(UpdateState::Downloading);
        let expected_result = update::UpdateEvent {
            event: update_event::Event::STATE_CHANGED.into(),
            state_changed: MessageField::some(update_event::StateChanged {
                new_state: update::update::State::DOWNLOADING.into(),
                special_fields: Default::default(),
            }),
            update_available: Default::default(),
            download_progress: Default::default(),
            special_fields: Default::default(),
        };

        let result = update::UpdateEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_update_event_from_download_progress() {
        let event = UpdateEvent::DownloadProgress(DownloadProgress {
            total_size: 20000,
            downloaded: 7000,
        });
        let expected_result = update::UpdateEvent {
            event: update_event::Event::DOWNLOAD_PROGRESS.into(),
            state_changed: Default::default(),
            update_available: Default::default(),
            download_progress: MessageField::some(update_event::DownloadProgress {
                progress: MessageField::some(update::update::DownloadProgress {
                    total_size: 20000,
                    downloaded: 7000,
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
            special_fields: Default::default(),
        };

        let result = update::UpdateEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_update_event_from_update_available() {
        let event = UpdateEvent::UpdateAvailable(VersionInfo {
            application: PatchInfo {
                version: "2.0.0".to_string(),
                platforms: Default::default(),
            },
            runtime: PatchInfo {
                version: "21.5.0".to_string(),
                platforms: Default::default(),
            },
        });
        let expected_result = update::UpdateEvent {
            event: update_event::Event::UPDATE_AVAILABLE.into(),
            state_changed: Default::default(),
            update_available: MessageField::some(update_event::UpdateAvailable {
                version_info: MessageField::some(update::update::VersionInfo {
                    application: MessageField::some(update::update::PatchInfo {
                        version: "2.0.0".to_string(),
                        platforms: Default::default(),
                        special_fields: Default::default(),
                    }),
                    runtime: MessageField::some(update::update::PatchInfo {
                        version: "21.5.0".to_string(),
                        platforms: Default::default(),
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
            download_progress: Default::default(),
            special_fields: Default::default(),
        };

        let result = update::UpdateEvent::from(&event);

        assert_eq!(expected_result, result);
    }
}
