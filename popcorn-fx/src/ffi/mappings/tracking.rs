use std::os::raw::c_char;

use popcorn_fx_core::core::media::tracking::TrackingEvent;

/// Type alias for the C-compatible authorization open function.
pub type AuthorizationOpenC = extern "C" fn(uri: *mut c_char) -> bool;

/// Type alias for the C-compatible tracking event callback function.
pub type TrackingEventCCallback = extern "C" fn(event: TrackingEventC);

/// Represents a C-compatible tracking event.
#[repr(C)]
#[derive(Debug)]
pub enum TrackingEventC {
    /// Authorization state change event.
    AuthorizationStateChanged(bool),
}

impl From<TrackingEvent> for TrackingEventC {
    fn from(value: TrackingEvent) -> Self {
        match value {
            TrackingEvent::AuthorizationStateChanged(e) => TrackingEventC::AuthorizationStateChanged(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use popcorn_fx_core::testing::init_logger;

    use super::*;

    #[test]
    fn test_from_tracking_event() {
        init_logger();

        let result = TrackingEventC::from(TrackingEvent::AuthorizationStateChanged(true));

        if let TrackingEventC::AuthorizationStateChanged(state) = result {
            assert_eq!(true, state);
        } else {
            assert!(false, "expected TrackingEventC::AuthorizationStateChanged, but got {:?} instead", result)
        }
    }
}