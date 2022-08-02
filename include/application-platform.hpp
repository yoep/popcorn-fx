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
struct PlatformC;

struct PlatformInfoC {
  /// The platform type
  PlatformType platform_type;
  /// The cpu architecture of the platform
  const char *arch;
};


extern "C" {

/// Disable the screensaver on the current platform
void disable_screensaver(Box<PlatformC> platform);

/// Enable the screensaver on the current platform
void enable_screensaver(Box<PlatformC> platform);

/// Retrieve the platform instance.
Box<PlatformC> new_platform_c();

/// Retrieve the platform information.
PlatformInfoC platform_info();

} // extern "C"
