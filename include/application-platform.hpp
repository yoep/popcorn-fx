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

/// PlatformInfo defines the info of the current platform
struct PlatformInfo {
  /// The platform type
  PlatformType platform_type;
  /// The cpu architecture of the platform
  const char *arch;
};


extern "C" {

/// Retrieve the platform information.
PlatformInfo platform_info();

} // extern "C"
