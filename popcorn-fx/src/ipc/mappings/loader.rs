use crate::ipc::proto::loader::loader_event;
use crate::ipc::proto::{loader, message};
use popcorn_fx_core::core::loader::{LoadingState, MediaLoaderEvent};
use protobuf::MessageField;

impl From<&MediaLoaderEvent> for loader::LoaderEvent {
    fn from(value: &MediaLoaderEvent) -> Self {
        let mut event = Self::new();

        match value {
            MediaLoaderEvent::LoadingStarted(handle, data) => {
                event.event = loader_event::Event::LOADING_STARTED.into();
                event.loading_started = MessageField::some(loader_event::LoadingStarted {
                    handle: MessageField::some(message::Handle::from(handle)),
                    url: data.url.clone(),
                    title: data.title.clone(),
                    thumbnail: data.thumbnail.clone(),
                    background: data.background.clone(),
                    quality: data.quality.clone(),
                    special_fields: Default::default(),
                });
            }
            MediaLoaderEvent::StateChanged(handle, state) => {
                event.event = loader_event::Event::STATE_CHANGED.into();
                event.state_changed = MessageField::some(loader_event::StateChanged {
                    handle: MessageField::some(message::Handle::from(handle)),
                    state: loader::loading::State::from(state).into(),
                    special_fields: Default::default(),
                });
            }
            MediaLoaderEvent::ProgressChanged(handle, progress) => {
                event.event = loader_event::Event::PROGRESS_CHANGED.into();
                event.progress_changed = MessageField::some(loader_event::ProgressChanged {
                    handle: MessageField::some(message::Handle::from(handle)),
                    progress: MessageField::some(loader::loading::Progress {
                        progress: progress.progress,
                        seeds: progress.seeds as u32,
                        peers: progress.peers as u32,
                        download_speed: progress.download_speed,
                        upload_speed: progress.upload_speed,
                        downloaded: progress.downloaded,
                        total_size: progress.total_size as u64,
                        special_fields: Default::default(),
                    }),
                    special_fields: Default::default(),
                });
            }
            MediaLoaderEvent::LoadingError(handle, _) => {
                event.event = loader_event::Event::LOADING_ERROR.into();
                event.loading_error = MessageField::some(loader_event::LoadingError {
                    handle: MessageField::some(message::Handle::from(handle)),
                    error: Default::default(),
                    special_fields: Default::default(),
                });
            }
        }

        event
    }
}

impl From<&LoadingState> for loader::loading::State {
    fn from(value: &LoadingState) -> Self {
        match value {
            LoadingState::Initializing => Self::INITIALIZING,
            LoadingState::Starting => Self::STARTING,
            LoadingState::RetrievingSubtitles => Self::RETRIEVING_SUBTITLES,
            LoadingState::DownloadingSubtitle => Self::DOWNLOADING_SUBTITLE,
            LoadingState::RetrievingMetadata => Self::RETRIEVING_METADATA,
            LoadingState::VerifyingFiles => Self::VERIFYING_FILES,
            LoadingState::Connecting => Self::CONNECTING,
            LoadingState::Downloading => Self::DOWNLOADING,
            LoadingState::DownloadFinished => Self::DOWNLOAD_FINISHED,
            LoadingState::Ready => Self::READY,
            LoadingState::Playing => Self::PLAYING,
            LoadingState::Cancelled => Self::CANCELLED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::proto::loader::loading;
    use popcorn_fx_core::core::loader::{LoadingHandle, LoadingProgress};

    #[test]
    fn test_loading_state_proto_from() {
        assert_eq!(
            loading::State::INITIALIZING,
            loader::loading::State::from(&LoadingState::Initializing)
        );
        assert_eq!(
            loading::State::STARTING,
            loader::loading::State::from(&LoadingState::Starting)
        );
        assert_eq!(
            loading::State::RETRIEVING_SUBTITLES,
            loader::loading::State::from(&LoadingState::RetrievingSubtitles)
        );
        assert_eq!(
            loading::State::DOWNLOADING_SUBTITLE,
            loader::loading::State::from(&LoadingState::DownloadingSubtitle)
        );
        assert_eq!(
            loading::State::RETRIEVING_METADATA,
            loader::loading::State::from(&LoadingState::RetrievingMetadata)
        );
        assert_eq!(
            loading::State::VERIFYING_FILES,
            loader::loading::State::from(&LoadingState::VerifyingFiles)
        );
        assert_eq!(
            loading::State::CONNECTING,
            loader::loading::State::from(&LoadingState::Connecting)
        );
        assert_eq!(
            loading::State::DOWNLOADING,
            loader::loading::State::from(&LoadingState::Downloading)
        );
        assert_eq!(
            loading::State::DOWNLOAD_FINISHED,
            loader::loading::State::from(&LoadingState::DownloadFinished)
        );
        assert_eq!(
            loading::State::READY,
            loader::loading::State::from(&LoadingState::Ready)
        );
        assert_eq!(
            loading::State::PLAYING,
            loader::loading::State::from(&LoadingState::Playing)
        );
        assert_eq!(
            loading::State::CANCELLED,
            loader::loading::State::from(&LoadingState::Cancelled)
        );
    }

    #[test]
    fn test_loader_event_from_progress_changed() {
        let handle = LoadingHandle::new();
        let event = MediaLoaderEvent::ProgressChanged(
            handle,
            LoadingProgress {
                progress: 13.5,
                seeds: 30,
                peers: 10,
                download_speed: 2048,
                upload_speed: 512,
                downloaded: 10000,
                total_size: 2800011,
            },
        );
        let expected_result = loader::LoaderEvent {
            event: loader_event::Event::PROGRESS_CHANGED.into(),
            loading_started: Default::default(),
            state_changed: Default::default(),
            progress_changed: MessageField::some(loader_event::ProgressChanged {
                handle: MessageField::some(message::Handle::from(&handle)),
                progress: MessageField::some(loading::Progress {
                    progress: 13.5,
                    seeds: 30,
                    peers: 10,
                    download_speed: 2048,
                    upload_speed: 512,
                    downloaded: 10000,
                    total_size: 2800011,
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            }),
            loading_error: Default::default(),
            special_fields: Default::default(),
        };

        let result = loader::LoaderEvent::from(&event);

        assert_eq!(expected_result, result);
    }
}
