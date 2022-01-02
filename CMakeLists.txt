cmake_minimum_required(VERSION 3.5)
project(PopcornTime LANGUAGES CXX)

message("")
message("The following CMAKE variables should be configured for this project:")
message("   CMAKE_PREFIX_PATH = The Qt install location (containing the directories bin, include & lib)")
message("   CMAKE_MODULE_PATH = The 'lib' directory within the bundled Qt compiler (containing the cmake files)")
message("")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

ADD_SUBDIRECTORY(application-ui/src/native)
ADD_SUBDIRECTORY(player-qt/src/native)
