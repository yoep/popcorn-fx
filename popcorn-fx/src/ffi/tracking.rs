use log::{error, info, trace};

use popcorn_fx_core::into_c_string;

use crate::ffi::AuthorizationOpenC;
use crate::PopcornFX;

#[no_mangle]
pub extern "C" fn register_tracking_authorization_open(popcorn_fx: &mut PopcornFX, callback: AuthorizationOpenC) {
    trace!("Registering new tracking authorization open callback from C");
    popcorn_fx.tracking_provider().register_open_authorization(Box::new(move |uri| {
        trace!("Calling tracker authorization open callback for {}", uri);
        callback(into_c_string(uri))
    }))
}

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

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;
    use std::sync::mpsc::channel;
    use std::time::Duration;

    use log::info;
    use tempfile::tempdir;

    use popcorn_fx_core::from_c_string;
    use popcorn_fx_core::testing::init_logger;

    use crate::test::new_instance;

    use super::*;

    extern "C" fn authorization_open(uri: *const c_char) -> bool {
        info!("Received authorization open uri {}", from_c_string(uri));
        true
    }

    #[test]
    fn test_register_tracking_authorization_open() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let mut instance = new_instance(temp_path);

        register_tracking_authorization_open(&mut instance, authorization_open);
    }

    #[test]
    fn test_tracking_authorize() {
        init_logger();
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();
        let (tx, rx) = channel();
        let mut instance = new_instance(temp_path);
        let properties = instance.settings().properties();
        let expected_uri = properties.tracker("trakt").unwrap().client.user_authorization_uri.clone();

        instance.tracking_provider().register_open_authorization(Box::new(move |uri| {
            tx.send(uri).unwrap();
            true
        }));

        tracking_authorize(&mut instance);

        let result = rx.recv_timeout(Duration::from_millis(200)).unwrap();

        assert!(result.starts_with(expected_uri.as_str()))
    }
}