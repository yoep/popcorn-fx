cmake_minimum_required(VERSION 3.5)
project(PopcornPlayer LANGUAGES CXX)

message("")
message("The following CMAKE variables should be configured for this project:")
message("   CMAKE_PREFIX_PATH = The compiler bundled with Qt")
message("   CMAKE_MODULE_PATH = The 'lib' directory within the bundled Qt compiler (containing the cmake files)")
message("")

ADD_SUBDIRECTORY(video-vlc-native/src/native)
