use log::{error, info, trace};
use popcorn_fx_core::core::block_in_place_runtime;
use popcorn_fx_core::into_c_string;

use crate::ffi::{AuthorizationOpenC, TrackingEventC, TrackingEventCCallback};
use crate::PopcornFX;

/// Registers a callback function to handle authorization URI openings from C code.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - The callback function to be registered.
#[no_mangle]
pub extern "C" fn register_tracking_authorization_open(
    popcorn_fx: &mut PopcornFX,
    callback: AuthorizationOpenC,
) {
    trace!("Registering new tracking authorization open callback from C");
    let tracking_provider = popcorn_fx.tracking_provider().clone();
    block_in_place_runtime(
        tracking_provider.register_open_authorization(Box::new(move |uri| {
            trace!("Calling tracker authorization open callback for {}", uri);
            callback(into_c_string(uri))
        })),
        popcorn_fx.runtime(),
    )
}

/// Registers a callback function to handle tracking provider events from C code.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
/// * `callback` - The callback function to be registered.
#[no_mangle]
pub extern "C" fn register_tracking_provider_callback(
    popcorn_fx: &mut PopcornFX,
    callback: TrackingEventCCallback,
) {
    trace!("Registering new tracking provider callback for C");
    let mut receiver = popcorn_fx.tracking_provider().subscribe();
    popcorn_fx.runtime().spawn(async move {
        while let Some(event) = receiver.recv().await {
            trace!("Invoking tracking event C for {:?}", event);
            callback(TrackingEventC::from((*event).clone()));
        }
    });
}

/// Checks if the current tracking provider is authorized.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
///
/// # Returns
///
/// Returns `true` if the tracking provider is authorized, otherwise `false`.
#[no_mangle]
pub extern "C" fn tracking_is_authorized(popcorn_fx: &mut PopcornFX) -> bool {
    trace!("Checking if the current tracker is authorized from C");
    let tracking = popcorn_fx.tracking_provider().clone();
    block_in_place_runtime(tracking.is_authorized(), popcorn_fx.runtime())
}

/// Initiates the authorization process with the tracking provider.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn tracking_authorize(popcorn_fx: &mut PopcornFX) {
    let tracking_service = popcorn_fx.tracking_provider().clone();
    popcorn_fx.runtime().spawn(async move {
        match tracking_service.authorize().await {
            Ok(_) => info!("Tracking provider authorization completed"),
            Err(e) => error!("Failed to authorize with tracking provider, {}", e),
        }
    });
}

/// Disconnects from the tracking provider.
///
/// # Arguments
///
/// * `popcorn_fx` - A mutable reference to a `PopcornFX` instance.
#[no_mangle]
pub extern "C" fn tracking_disconnect(popcorn_fx: &mut PopcornFX) {
    trace!("Disconnecting tracker");
    let tracking_service = popcorn_fx.tracking_provider().clone();
    popcorn_fx
        .runtime()
        .spawn(async move { tracking_service.disconnect().await });
}

/// Disposes a tracking event value.
///
/// # Arguments
///
/// * `event` - The tracking event to be disposed.
#[no_mangle]
pub extern "C" fn dispose_tracking_event_value(event: TrackingEventC) {
    trace!("Disposing {:?}", event);
    drop(event);
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use log::info;
    use reqwest::Client;
    use tempfile::tempdir;
    use url::Url;

    use popcorn_fx_core::core::config::Tracker;
    use popcorn_fx_core::core::{block_in_place, block_in_place_runtime};
    use popcorn_fx_core::{assert_timeout_eq, from_c_string, init_logger};

    use crate::test::new_instance;

    use super::*;

    extern "C" fn authorization_open(uri: *mut c_char) -> bool {
        info!("Received authorization open uri {}", from_c_string(uri));
        true
    }

    extern "C" fn tracking_event_c_callback(event: TrackingEventC) {
        info!("Received tracking event C callback {:?}", event)
    }

    #[test]
    fn test_register_tracking_authorization_open() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        register_tracking_authorization_open(&mut instance, authorization_open);
    }

    #[test]
    fn test_register_tracking_provider_callback() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        register_tracking_provider_callback(&mut instance, tracking_event_c_callback);
    }

    #[test]
    fn test_tracking_is_authorized() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        let result = tracking_is_authorized(&mut instance);

        assert!(!result, "expected the tracker to not have been authorized");
    }

    #[test]
    fn test_tracking_authorize() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut instance = new_instance(temp_path);
        let properties = instance.settings().properties();
        let expected_uri = properties
            .tracker("trakt")
            .unwrap()
            .client
            .user_authorization_uri
            .clone();

        let tracking_provider = instance.tracking_provider().clone();
        block_in_place_runtime(
            tracking_provider.register_open_authorization(Box::new(move |url| {
                // execute a callback to stop the authorization process
                let client = Client::new();
                let auth_uri = Url::parse(url.as_str()).unwrap();

                let (_, redirect_uri) = auth_uri
                    .query_pairs()
                    .find(|(key, _)| key == "redirect_uri")
                    .expect("expected the redirect uri to have been present");

                let mut uri =
                    Url::parse(redirect_uri.as_ref()).expect("expected a valid redirect uri");
                let uri = uri
                    .query_pairs_mut()
                    .append_pair("code", "someRandomCode")
                    .append_pair("state", "SomeState")
                    .finish();

                block_in_place(client.get(uri.as_str()).send()).unwrap();

                tx.send(url).unwrap();
                true
            })),
            instance.runtime(),
        );

        tracking_authorize(&mut instance);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();
        assert!(result.starts_with(expected_uri.as_str()))
    }

    #[test]
    fn test_tracking_disconnect() {
        init_logger!();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);
        let settings = instance.settings().clone();
        block_in_place_runtime(
            settings.update_tracker(
                "trakt",
                Tracker {
                    access_token: "MyAccessToken".to_string(),
                    expires_in: None,
                    refresh_token: None,
                    scopes: None,
                },
            ),
            instance.runtime(),
        );

        let tracking = instance.tracking_provider().clone();
        let result = block_in_place_runtime(tracking.is_authorized(), instance.runtime());
        assert_eq!(true, result, "expected the tracker to have been authorized");
        tracking_disconnect(&mut instance);

        block_in_place_runtime(
            async {
                assert_timeout_eq!(
                    Duration::from_millis(500),
                    false,
                    instance.tracking_provider().is_authorized().await
                )
            },
            instance.runtime(),
        );
    }
}
