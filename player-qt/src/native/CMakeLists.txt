cmake_minimum_required(VERSION 3.5)

message(STATUS "Using Qt CMAKE_PREFIX_PATH: ${CMAKE_PREFIX_PATH}")
message(STATUS "Using Qt CMAKE_MODULE_PATH: ${CMAKE_MODULE_PATH}")

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake ${CMAKE_MODULE_PATH})
set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(QT NAMES Qt6 Qt5 COMPONENTS Widgets Quick QuickWidgets REQUIRED)
find_package(Qt${QT_VERSION_MAJOR} COMPONENTS Widgets Quick QuickWidgets REQUIRED)

find_package(LIBVLC REQUIRED)
include_directories("${LIBVLC_INCLUDE_DIR}")

file(GLOB LIBVLC_LIBS
        ${LIBVLC_INCLUDE_DIR}/vlc/*.h)

file(GLOB SOURCES_RUNNER
        ${CMAKE_SOURCE_DIR}/shared/*.cpp
        ${CMAKE_SOURCE_DIR}/shared/*.h
        ${CMAKE_CURRENT_SOURCE_DIR}/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/*.h
        ${CMAKE_CURRENT_SOURCE_DIR}/*.ui
        ${CMAKE_CURRENT_SOURCE_DIR}/*.qrc
        ${CMAKE_CURRENT_SOURCE_DIR}/widgets/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/widgets/*.h
        ${CMAKE_CURRENT_SOURCE_DIR}/widgets/*.ui
        ${CMAKE_CURRENT_SOURCE_DIR}/player/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/player/*.h
        ${LIBVLC_LIBS})

# exclude the main.cpp from the library
set(SOURCES ${SOURCES_RUNNER})
list(REMOVE_ITEM SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/main.cpp")
message("Including: ${SOURCES}")

add_library(PopcornPlayer SHARED ${SOURCES})
add_executable(PopcornPlayerRunner ${SOURCES_RUNNER})

target_link_libraries(PopcornPlayer
        Qt${QT_VERSION_MAJOR}::Widgets Qt${QT_VERSION_MAJOR}::Quick Qt${QT_VERSION_MAJOR}::QuickWidgets
        ${LIBVLC_LIBRARY} ${LIBVLCCORE_LIBRARY}
        -lpthread)
target_link_libraries(PopcornPlayerRunner
        Qt${QT_VERSION_MAJOR}::Widgets Qt${QT_VERSION_MAJOR}::Quick Qt${QT_VERSION_MAJOR}::QuickWidgets
        ${LIBVLC_LIBRARY} ${LIBVLCCORE_LIBRARY}
        -lpthread)
