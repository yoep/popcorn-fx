use crate::popcorn::fx::popcorn_fx::PopcornFX;

pub mod popcorn;

/// Create a new PopcornFX instance.
/// It returns a reference to the popcorn FX instance.
#[no_mangle]
pub extern "C" fn new_instance() -> Box<PopcornFX> {
    return Box::new(PopcornFX::new());
}