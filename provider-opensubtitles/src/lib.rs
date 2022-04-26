use std::ptr;

use crate::popcorn::fx::subtitle::model::Subtitle;
use crate::popcorn::fx::subtitle::service::SubtitleService;

pub mod popcorn;

#[no_mangle]
pub extern "C" fn subtitle_service_new() -> *mut SubtitleService {
    Box::into_raw(Box::new(SubtitleService::new()))
}

#[no_mangle]
pub unsafe extern "C" fn subtitle_service_active_subtitle(ptr: *mut SubtitleService) -> *mut &'static Subtitle {
    assert!(!ptr.is_null());
    let service = &*ptr;

    return match service.active_subtitle() {
        Some(subtitle) => Box::into_raw(Box::new(subtitle)),
        None => ptr::null_mut()
    };
}

#[no_mangle]
pub unsafe extern "C" fn subtitle_service_release(ptr: *mut SubtitleService) {
    assert!(!ptr.is_null());
    drop(Box::from_raw(ptr))
}