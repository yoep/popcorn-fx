use std::ffi::CStr;
use std::os::raw::c_char;

use derive_more::Display;
use log::error;

use popcorn_fx_core::core::players::{Player, PlayerState};

#[repr(C)]
#[derive(Debug, Display)]
#[display(fmt = "id: {}", "self.id()")]
pub struct PlayerC {
    pub id: *const c_char,
    pub name: *const c_char,
    pub description: *const c_char,
}

impl PlayerC {
    fn from_c_string_slice(ptr: *const c_char) -> &'static str {
        if !ptr.is_null() {
            let slice = unsafe { CStr::from_ptr(ptr).to_bytes() };

            return std::str::from_utf8(slice)
                .unwrap_or_else(|e| {
                    error!("Failed to read player value, using empty string instead ({})", e);
                    ""
                });
        }

        ""
    }
}

impl Player for PlayerC {
    fn id(&self) -> &str {
        Self::from_c_string_slice(self.id)
    }

    fn name(&self) -> &str {
        Self::from_c_string_slice(self.name)
    }

    fn description(&self) -> &str {
        Self::from_c_string_slice(self.description)
    }

    fn graphic_resource(&self) -> Vec<u8> {
        todo!()
    }

    fn state(&self) -> &PlayerState {
        todo!()
    }
}