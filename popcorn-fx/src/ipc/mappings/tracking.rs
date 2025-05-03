use crate::ipc::proto::tracking;
use crate::ipc::proto::tracking::tracking_provider::{authorization_error, AuthorizationState};
use crate::ipc::proto::tracking::{tracking_provider, tracking_provider_event};
use popcorn_fx_core::core::media::tracking::{AuthorizationError, TrackingEvent};
use protobuf::MessageField;

impl From<&TrackingEvent> for tracking::TrackingProviderEvent {
    fn from(value: &TrackingEvent) -> Self {
        let mut event = Self::new();

        match value {
            TrackingEvent::AuthorizationStateChanged(state) => {
                let state = if *state {
                    AuthorizationState::AUTHORIZED.into()
                } else {
                    AuthorizationState::UNAUTHORIZED.into()
                };

                event.event = tracking_provider_event::Event::AUTHORIZATION_STATE_CHANGED.into();
                event.authorization_state_changed =
                    MessageField::some(tracking_provider_event::AuthorizationStateChanged {
                        state,
                        special_fields: Default::default(),
                    });
            }
            TrackingEvent::OpenAuthorization(authorization_uri) => {
                event.event = tracking_provider_event::Event::OPEN_AUTHORIZATION_URI.into();
                event.open_authorization_uri =
                    MessageField::some(tracking_provider_event::OpenAuthorizationUri {
                        uri: authorization_uri.to_string(),
                        special_fields: Default::default(),
                    });
            }
        }

        event
    }
}

impl From<&AuthorizationError> for tracking_provider::AuthorizationError {
    fn from(value: &AuthorizationError) -> Self {
        match value {
            AuthorizationError::CsrfFailure => Self {
                type_: authorization_error::Type::CSRF_FAILURE.into(),
                special_fields: Default::default(),
            },
            AuthorizationError::AuthorizationCode => Self {
                type_: authorization_error::Type::AUTHORIZATION_CODE.into(),
                special_fields: Default::default(),
            },
            AuthorizationError::Token => Self {
                type_: authorization_error::Type::TOKEN.into(),
                special_fields: Default::default(),
            },
            AuthorizationError::AuthorizationTimeout => Self {
                type_: authorization_error::Type::AUTHORIZATION_TIMEOUT.into(),
                special_fields: Default::default(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use url::Url;

    #[test]
    fn test_tracking_provider_event_proto_from_authorization_state_changed() {
        let event = TrackingEvent::AuthorizationStateChanged(true);
        let expected_result = tracking::TrackingProviderEvent {
            event: tracking_provider_event::Event::AUTHORIZATION_STATE_CHANGED.into(),
            authorization_state_changed: MessageField::some(
                tracking_provider_event::AuthorizationStateChanged {
                    state: AuthorizationState::AUTHORIZED.into(),
                    special_fields: Default::default(),
                },
            ),
            open_authorization_uri: Default::default(),
            special_fields: Default::default(),
        };

        let result = tracking::TrackingProviderEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_tracking_provider_event_proto_from_open_authorization_uri() {
        let uri = "https://trackingprovider.com/authorize";
        let event = TrackingEvent::OpenAuthorization(Url::from_str(uri).unwrap());
        let expected_result = tracking::TrackingProviderEvent {
            event: tracking_provider_event::Event::OPEN_AUTHORIZATION_URI.into(),
            authorization_state_changed: Default::default(),
            open_authorization_uri: MessageField::some(
                tracking_provider_event::OpenAuthorizationUri {
                    uri: uri.to_string(),
                    special_fields: Default::default(),
                },
            ),
            special_fields: Default::default(),
        };

        let result = tracking::TrackingProviderEvent::from(&event);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_authorization_error_from_csrf_failure() {
        let err = AuthorizationError::CsrfFailure;

        let result = tracking_provider::AuthorizationError::from(&err);

        assert_eq!(
            authorization_error::Type::CSRF_FAILURE,
            result.type_.unwrap()
        );
    }

    #[test]
    fn test_authorization_error_from_token() {
        let err = AuthorizationError::Token;

        let result = tracking_provider::AuthorizationError::from(&err);

        assert_eq!(authorization_error::Type::TOKEN, result.type_.unwrap());
    }

    #[test]
    fn test_authorization_error_from_authorization_code() {
        let err = AuthorizationError::AuthorizationCode;

        let result = tracking_provider::AuthorizationError::from(&err);

        assert_eq!(
            authorization_error::Type::AUTHORIZATION_CODE,
            result.type_.unwrap()
        );
    }

    #[test]
    fn test_authorization_error_from_authorization_timeout() {
        let err = AuthorizationError::AuthorizationTimeout;

        let result = tracking_provider::AuthorizationError::from(&err);

        assert_eq!(
            authorization_error::Type::AUTHORIZATION_TIMEOUT,
            result.type_.unwrap()
        );
    }
}
