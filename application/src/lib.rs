use log::info;

use crate::popcorn::fx::popcorn_fx::PopcornFX;

pub mod popcorn;


/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [delete_popcorn_fx].
#[no_mangle]
pub extern "C" fn new_popcorn_fx() -> Box<PopcornFX> {
    return Box::new(PopcornFX::new());
}

/// Delete the PopcornFX instance in a safe way.
#[no_mangle]
pub extern "C" fn delete_popcorn_fx(popcorn_fx: Box<PopcornFX>) {
    info!("Deleting Popcorn FX");
    drop(popcorn_fx)
}
