#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


/// The platform type
enum class PlatformType {
  /// The windows platform
  Windows = 0,
  /// The macos platform
  MacOs = 1,
  /// The linux platform
  Linux = 2,
};

template<typename T = void>
struct Box;

/// The actions for the current platform.
/// This is a C wrapper around the rust struct.
struct PlatformC;

struct PlatformInfoC {
  /// The platform type
  PlatformType platform_type;
  /// The cpu architecture of the platform
  const char *arch;
};


extern "C" {

/// Drop the platform instance.
void delete_platform_c(Box<PlatformC> platform);

/// Disable the screensaver on the current platform
void disable_screensaver(PlatformC *platform);

/// Enable the screensaver on the current platform
void enable_screensaver(PlatformC *platform);

/// Retrieve the platform instance.
/// The caller will become the owner of the instance and is responsible for the memory management of it.
/// The instance can be safely deleted by using [delete_platform_c()]
Box<PlatformC> new_platform_c();

/// Retrieve the platform information.
PlatformInfoC platform_info();

} // extern "C"
