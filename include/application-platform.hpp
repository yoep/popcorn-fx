#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


enum class PlatformType {
  Windows = 0,
  MacOs = 1,
  Linux = 2,
};

struct PlatformInfo {
  PlatformType platform_type;
  const char *arch;
};


extern "C" {

PlatformInfo platform_info();

} // extern "C"
