#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


template<typename T = void>
struct Box;

/// The Popcorn FX struct contains the main controller logic of popcorn.
struct PopcornFX;


extern "C" {

/// Create a new PopcornFX instance.
/// It returns a reference to the popcorn FX instance.
Box<PopcornFX> new_instance();

} // extern "C"
