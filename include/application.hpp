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

/// Delete the PopcornFX instance in a safe way.
void delete_popcorn_fx(Box<PopcornFX> popcorn_fx);

/// Create a new PopcornFX instance.
/// The caller will become responsible for managing the memory of the struct.
/// The instance can be safely deleted by using [delete_popcorn_fx].
Box<PopcornFX> new_popcorn_fx();

} // extern "C"
