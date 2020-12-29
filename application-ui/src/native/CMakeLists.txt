cmake_minimum_required(VERSION 3.5)

message(STATUS "Building the Popcorn Keys library")

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake ${CMAKE_MODULE_PATH})
set(CMAKE_INCLUDE_CURRENT_DIR ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# global sources which are not OS specific
file(GLOB SOURCES
        ${CMAKE_SOURCE_DIR}/shared/*.cpp
        ${CMAKE_SOURCE_DIR}/shared/*.h
        ${CMAKE_CURRENT_SOURCE_DIR}/*.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/*.h)

# add OS specific files
if (UNIX AND NOT APPLE)
    message(STATUS "Including X11 libraries")
    find_package(X11 REQUIRED)

    link_libraries(${X11_LIBRARIES})
    include_directories(${X11_INCLUDE_DIR})

    message(STATUS "Including GTK-3")
    find_package(PkgConfig REQUIRED)

    pkg_check_modules(GTK3 REQUIRED gtk+-3.0)
    include_directories(${GTK3_INCLUDE_DIRS})
    link_libraries(${GTK3_LIBRARY_DIRS})

    message(STATUS "Adding linux sources to the sources list")
    file(GLOB LINUX_SOURCES
            ${CMAKE_CURRENT_SOURCE_DIR}/linux/*.cpp
            ${CMAKE_CURRENT_SOURCE_DIR}/linux/*.h)

    list(APPEND SOURCES ${LINUX_SOURCES})
elseif (WIN32 OR WIN64)
    file(GLOB WINDOWS_SOURCES
            ${CMAKE_CURRENT_SOURCE_DIR}/windows/*.cpp
            ${CMAKE_CURRENT_SOURCE_DIR}/windows/*.h)

    list(APPEND SOURCES ${WINDOWS_SOURCES})
endif ()

set(SOURCES_LIB ${SOURCES})
list(REMOVE_ITEM SOURCES_LIB "${CMAKE_CURRENT_SOURCE_DIR}/main.cpp")

message("Including: ${SOURCES_LIB}")
add_library(PopcornKeys SHARED ${SOURCES_LIB})
add_executable(PopcornKeysRunner ${SOURCES})

# link specific OS libs
if (UNIX AND NOT APPLE)
    target_link_libraries(PopcornKeys
            ${GTK3_LIBRARIES}
            -lpthread)
    target_link_libraries(PopcornKeysRunner
            ${GTK3_LIBRARIES}
            -lpthread)
elseif (WIN32 OR WIN64)

endif ()
