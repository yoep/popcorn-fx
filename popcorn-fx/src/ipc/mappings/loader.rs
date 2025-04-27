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
