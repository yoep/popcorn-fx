cmake_minimum_required(VERSION 3.5)

message(STATUS "Using Qt CMAKE_PREFIX_PATH: ${CMAKE_PREFIX_PATH}")

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake ${CMAKE_MODULE_PATH})
set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt5 COMPONENTS Widgets REQUIRED)

find_package(LIBVLC REQUIRED)
include_directories("${LIBVLC_INCLUDE_DIR}")

file(GLOB LIBVLC_LIBS
        ${LIBVLC_INCLUDE_DIR}/vlc/*.h)

file(GLOB SOURCES
        ${CMAKE_CURRENT_SOURCE_DIR}/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/*.h
        ${LIBVLC_LIBS})
message("Including: ${SOURCES}")

add_library(PopcornDesktopPlayer SHARED ${SOURCES})
add_executable(PopcornDesktopPlayerRunner ${SOURCES})

target_link_libraries(PopcornDesktopPlayer Qt5::Widgets ${LIBVLC_LIBRARY} ${LIBVLCCORE_LIBRARY})
target_link_libraries(PopcornDesktopPlayerRunner Qt5::Widgets ${LIBVLC_LIBRARY} ${LIBVLCCORE_LIBRARY})
