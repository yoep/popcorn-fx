use std::os::raw::c_char;

pub type AuthorizationOpenC = extern "C" fn(uri: *const c_char) -> bool;