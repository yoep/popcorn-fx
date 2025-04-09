use crate::ipc::proto::events;
use crate::ipc::proto::events::event::{EventType, PlaybackStateChanged, PlayerChanged};
use crate::ipc::proto::player::player;
use popcorn_fx_core::core::event::Event;
use protobuf::MessageField;

impl From<&Event> for events::Event {
    fn from(value: &Event) -> Self {
        let mut event = events::Event::new();

        match value {
            Event::PlayerChanged(e) => {
                event.type_ = EventType::PLAYER_CHANGED.into();
                event.player_changed = MessageField::some(PlayerChanged {
                    old_player_id: e.old_player_id.clone(),
                    new_player_id: e.new_player_id.clone(),
                    new_player_name: e.new_player_name.clone(),
                    special_fields: Default::default(),
                });
            }
            Event::PlayerStarted(_) => {
                event.type_ = EventType::PLAYER_STARTED.into();
            }
            Event::PlayerStopped(_) => {
                event.type_ = EventType::PLAYER_STOPPED.into();
            }
            Event::PlaybackStateChanged(e) => {
                event.type_ = EventType::PLAYBACK_STATE_CHANGED.into();
                event.playback_state_changed = MessageField::some(PlaybackStateChanged {
                    new_state: player::State::from(e).into(),
                    special_fields: Default::default(),
                });
            }
            Event::WatchStateChanged(_, _) => {
                event.type_ = EventType::WATCH_STATE_CHANGED.into();
            }
            Event::LoadingStarted => {
                event.type_ = EventType::LOADING_STARTED.into();
            }
            Event::LoadingCompleted => {
                event.type_ = EventType::LOADING_COMPLETED.into();
            }
            Event::TorrentDetailsLoaded(_) => {
                event.type_ = EventType::TORRENT_DETAILS_LOADED.into();
            }
            Event::ClosePlayer => {
                event.type_ = EventType::CLOSE_PLAYER.into();
            }
        }

        event
    }
}
