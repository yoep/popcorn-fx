use crate::ipc::proto::stream;
use popcorn_fx_core::core::stream::{Error, ServerStream, StreamEvent, StreamState, StreamStats};
use protobuf::{EnumOrUnknown, MessageField};

impl From<&StreamEvent> for stream::stream::StreamEvent {
    fn from(value: &StreamEvent) -> Self {
        match value {
            StreamEvent::StateChanged(state) => Self {
                type_: stream::stream::stream_event::Type::STATE_CHANGED.into(),
                filename: String::new(),
                state: Some(EnumOrUnknown::from(stream::stream::StreamState::from(
                    *state,
                ))),
                stats: Default::default(),
                special_fields: Default::default(),
            },
            StreamEvent::StatsChanged(stats) => Self {
                type_: stream::stream::stream_event::Type::STATS_CHANGED.into(),
                filename: String::new(),
                state: None,
                stats: MessageField::some(stream::stream::StreamStats::from(*stats)),
                special_fields: Default::default(),
            },
        }
    }
}

impl From<ServerStream> for stream::ServerStream {
    fn from(value: ServerStream) -> Self {
        Self {
            url: value.url.to_string(),
            filename: value.filename,
            special_fields: Default::default(),
        }
    }
}

impl From<StreamState> for stream::stream::StreamState {
    fn from(value: StreamState) -> Self {
        match value {
            StreamState::Preparing => Self::PREPARING,
            StreamState::Streaming => Self::STREAMING,
            StreamState::Stopped => Self::STOPPED,
        }
    }
}

impl From<StreamStats> for stream::stream::StreamStats {
    fn from(value: StreamStats) -> Self {
        Self {
            progress: value.progress,
            connections: value.connections as u64,
            download_speed: value.download_speed,
            upload_speed: value.upload_speed,
            downloaded: value.downloaded as u64,
            total_size: value.total_size as u64,
            special_fields: Default::default(),
        }
    }
}

impl From<Error> for stream::stream::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::AlreadyExists(message) => stream::stream::Error {
                type_: stream::stream::error::Type::ALREADY_EXISTS.into(),
                already_exists: MessageField::some(stream::stream::error::AlreadyExists {
                    message,
                    special_fields: Default::default(),
                }),
                not_found: Default::default(),
                io: Default::default(),
                parse: Default::default(),
                special_fields: Default::default(),
            },
            Error::NotFound(message) => stream::stream::Error {
                type_: stream::stream::error::Type::NOT_FOUND.into(),
                already_exists: Default::default(),
                not_found: MessageField::some(stream::stream::error::NotFound {
                    message,
                    special_fields: Default::default(),
                }),
                io: Default::default(),
                parse: Default::default(),
                special_fields: Default::default(),
            },
            Error::InvalidState => stream::stream::Error {
                type_: stream::stream::error::Type::INVALID_STATE.into(),
                already_exists: Default::default(),
                not_found: Default::default(),
                io: Default::default(),
                parse: Default::default(),
                special_fields: Default::default(),
            },
            Error::InvalidRange => stream::stream::Error {
                type_: stream::stream::error::Type::INVALID_RANGE.into(),
                already_exists: Default::default(),
                not_found: Default::default(),
                io: Default::default(),
                parse: Default::default(),
                special_fields: Default::default(),
            },
            Error::Parse(message) => stream::stream::Error {
                type_: stream::stream::error::Type::PARSE.into(),
                already_exists: Default::default(),
                not_found: Default::default(),
                io: Default::default(),
                parse: MessageField::some(stream::stream::error::ParseError {
                    message,
                    special_fields: Default::default(),
                }),
                special_fields: Default::default(),
            },
            Error::Io(message) => stream::stream::Error {
                type_: stream::stream::error::Type::IO.into(),
                already_exists: Default::default(),
                not_found: Default::default(),
                io: MessageField::some(stream::stream::error::IoError {
                    message: message.to_string(),
                    special_fields: Default::default(),
                }),
                parse: Default::default(),
                special_fields: Default::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use url::Url;

    #[test]
    fn test_server_stream_from() {
        let stream = ServerStream {
            url: Url::parse("http://localhost:8080/test.mkv").unwrap(),
            filename: "test.mkv".to_string(),
        };
        let expected_result = stream::ServerStream {
            url: "http://localhost:8080/test.mkv".to_string(),
            filename: "test.mkv".to_string(),
            special_fields: Default::default(),
        };

        let result = stream.into();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_from_stream_state() {
        assert_eq!(
            stream::stream::StreamState::PREPARING,
            StreamState::Preparing.into()
        );
        assert_eq!(
            stream::stream::StreamState::STREAMING,
            StreamState::Streaming.into()
        );
        assert_eq!(
            stream::stream::StreamState::STOPPED,
            StreamState::Stopped.into()
        );
    }

    #[test]
    fn test_from_stream_stats() {
        let stats = StreamStats {
            progress: 0.5,
            connections: 10,
            download_speed: 1000,
            upload_speed: 2000,
            downloaded: 10000,
            total_size: 20000,
        };
        let expected_result = stream::stream::StreamStats {
            progress: 0.5,
            connections: 10,
            download_speed: 1000,
            upload_speed: 2000,
            downloaded: 10000,
            total_size: 20000,
            special_fields: Default::default(),
        };

        let result = stats.into();

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_from_error() {
        let message = "Foo";
        let err = Error::AlreadyExists(message.to_string());
        let result = stream::stream::Error::from(err);
        assert_eq!(
            stream::stream::error::Type::ALREADY_EXISTS,
            result.type_.unwrap()
        );
        assert_eq!(
            Some(message),
            result.already_exists.as_ref().map(|e| e.message.as_str())
        );

        let message = "Bar";
        let err = Error::NotFound(message.to_string());
        let result = stream::stream::Error::from(err);
        assert_eq!(
            stream::stream::error::Type::NOT_FOUND,
            result.type_.unwrap()
        );
        assert_eq!(
            Some(message),
            result.not_found.as_ref().map(|e| e.message.as_str())
        );

        let err = Error::InvalidState;
        let result = stream::stream::Error::from(err);
        assert_eq!(
            stream::stream::error::Type::INVALID_STATE,
            result.type_.unwrap()
        );

        let err = Error::InvalidRange;
        let result = stream::stream::Error::from(err);
        assert_eq!(
            stream::stream::error::Type::INVALID_RANGE,
            result.type_.unwrap()
        );

        let message = "FooBar";
        let err = Error::Parse(message.to_string());
        let result = stream::stream::Error::from(err);
        assert_eq!(stream::stream::error::Type::PARSE, result.type_.unwrap());
        assert_eq!(
            Some(message),
            result.parse.as_ref().map(|e| e.message.as_str())
        );

        let message = "Ipsum";
        let err = Error::Io(io::Error::new(io::ErrorKind::Other, message));
        let result = stream::stream::Error::from(err);
        assert_eq!(stream::stream::error::Type::IO, result.type_.unwrap());
        assert_eq!(
            Some(message),
            result.io.as_ref().map(|e| e.message.as_str())
        );
    }
}
