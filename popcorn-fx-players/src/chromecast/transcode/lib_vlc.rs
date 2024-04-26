use std::{env, ptr};

use derive_more::Display;
use libloading::{Library, Symbol};
use log::{debug, error, trace};

/// A wrapper around a `*mut libc::c_void` which provides a [Send] safety between threads.
#[derive(Debug, Clone, Copy)]
pub struct LibvlcInstanceT<T>(pub T);

impl<T> LibvlcInstanceT<T> {
    /// Creates a new `LibvlcInstanceT` instance with the provided instance.
    pub fn new(instance: T) -> Self {
        Self {
            0: instance,
        }
    }
}

// Safety: LibvlcInstanceT is safe to be sent between threads.
unsafe impl<T> Send for LibvlcInstanceT<T> {}

/// Represents a libvlc instance.
#[allow(non_camel_case_types)]
pub type libvlc_instance_t = *mut libc::c_void;
/// Represents a libvlc media player.
#[allow(non_camel_case_types)]
pub type libvlc_media_player_t = *mut libc::c_void;
/// Represents a libvlc media.
#[allow(non_camel_case_types)]
pub type libvlc_media_t = *mut libc::c_void;

/// Represents the libvlc_new function signature.
#[allow(non_camel_case_types)]
pub type libvlc_new = extern "C" fn(argc: *const i32, argv: *const *const libc::c_char) -> libvlc_instance_t;
/// Represents the libvlc_media_player_new function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_player_new = extern "C" fn(instance: libvlc_instance_t) -> libvlc_media_player_t;
/// Represents the libvlc_media_player_release function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_player_release = extern "C" fn(media: libvlc_media_player_t);
/// Represents the libvlc_media_new_location function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_new_location = extern "C" fn(instance: libvlc_instance_t, path: *const libc::c_char) -> libvlc_media_t;
/// Represents the libvlc_media_add_option function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_add_option = extern "C" fn(media: libvlc_media_t, option: *const libc::c_char);
/// Represents the libvlc_media_release function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_release = extern "C" fn(media: libvlc_media_t);
/// Represents the libvlc_media_player_set_media function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_player_set_media = extern "C" fn(media_player: libvlc_media_player_t, media: libvlc_media_t);
/// Represents the libvlc_media_player_play function signature.
/// 
/// # Returns
/// 
/// 0 if playback started (and was already started), or -1 on error.
#[allow(non_camel_case_types)]
pub type libvlc_media_player_play = extern "C" fn(media_player: libvlc_media_player_t) -> libc::c_int;
/// Represents the libvlc_media_player_stop function signature.
#[allow(non_camel_case_types)]
pub type libvlc_media_player_stop = extern "C" fn(media_player: libvlc_media_player_t);

/// Represents a handle to the VLC library and associated plugins.
#[derive(Debug, Display)]
#[display(fmt = "lib: {}, plugins: {}", lib_path, plugin_path)]
pub struct LibraryHandle {
    lib_path: String,
    plugin_path: String,
    libvlc: Library,
    libvlc_core: Library,
}

impl LibraryHandle {
    /// Creates a new `LibraryHandle` instance.
    ///
    /// # Arguments
    ///
    /// * `lib_path` - The path to the VLC library.
    /// * `plugin_path` - The path to the VLC plugins.
    /// * `libvlc` - The handle to the VLC library.
    /// * `libvlc_core` - The handle to the VLC core library.
    ///
    /// # Returns
    ///
    /// A new `LibraryHandle` instance.
    pub fn new(lib_path: impl Into<String>,
               plugin_path: impl Into<String>,
               libvlc: Library,
               libvlc_core: Library) -> Self {
        Self {
            lib_path: lib_path.into(),
            plugin_path: plugin_path.into(),
            libvlc,
            libvlc_core,
        }
    }

    /// Gets a symbol from the VLC library.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol.
    ///
    /// # Returns
    ///
    /// A `Result` containing the symbol if found, or an error.
    pub fn get<T>(&self, name: &[u8]) -> Result<Symbol<T>, libloading::Error> {
        unsafe { self.libvlc.get(name) }
    }

    /// Creates a new libvlc instance.
    ///
    /// # Returns
    ///
    /// An `Option` containing the new libvlc instance if successful, otherwise `None`.
    pub fn libvlc_instance(&self) -> Option<libvlc_instance_t> {
        // always make sure that the VLC_PLUGIN_PATH environment variable is set to the correct path
        // before trying to create a new libvlc instance, as it will fail if it is not set
        // and always return a null pointer in that case
        env::set_var("VLC_PLUGIN_PATH", self.plugin_path.as_str());
        match self.get::<libvlc_new>(b"libvlc_new\0") {
            Ok(native_fn) => {
                trace!("Invoking libvlc_new with no arguments");
                let instance = native_fn(0 as *const i32, ptr::null());

                trace!("Invocation of libvlc_new result: {:?}", instance);
                if !instance.is_null() {
                    debug!("Created new libvlc instance {:?}", instance);
                    return Some(instance);
                } else {
                    error!("Failed to initialize libvlc instance");
                }
            }
            Err(e) => error!("Failed to load libvlc_new symbol, {}", e),
        };

        None
    }
}