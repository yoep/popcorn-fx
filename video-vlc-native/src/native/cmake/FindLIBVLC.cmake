FIND_PATH(
        LIBVLC_INCLUDE_DIR vlc/vlc.h
        HINTS "$ENV{LIBVLC_INCLUDE_PATH}"
        PATHS
        "${CMAKE_CURRENT_SOURCE_DIR}/libvlc"
        "${CMAKE_CURRENT_SOURCE_DIR}/libvlc/vlc"
)

# fix the CMAKE_FIND_LIBRARY_SUFFIXES for windows as the list is incorrect
# the original list contains ".ddl.a",".a",".lib"
if (WIN32)
    SET(CMAKE_FIND_LIBRARY_SUFFIXES ".dll" ".a" ".lib")
endif ()

# check if the ARM flag is defined
# if this is the case, we're only going to check the linux-arm directory for the VLC libraries
if (ARM)
    FIND_LIBRARY(
            LIBVLC_LIBRARY
            NAMES vlc libvlc
            HINTS "$ENV{LIBVLC_LIBRARY_PATH}"
            PATHS
            "${PROJECT_SOURCE_DIR}/assets/linux-arm/vlc"
    )
    FIND_LIBRARY(
            LIBVLCCORE_LIBRARY
            NAMES vlccore libvlccore
            HINTS "$ENV{LIBVLC_LIBRARY_PATH}"
            PATHS
            "${PROJECT_SOURCE_DIR}/assets/linux-arm/vlc"
    )
else ()
    FIND_LIBRARY(
            LIBVLC_LIBRARY
            NAMES vlc libvlc
            HINTS "$ENV{LIBVLC_LIBRARY_PATH}"
            PATHS
            "${PROJECT_SOURCE_DIR}/assets/linux/vlc"
            "${PROJECT_SOURCE_DIR}/assets/linux-arm/vlc"
            "${PROJECT_SOURCE_DIR}/assets/maxosx/vlc"
            "${PROJECT_SOURCE_DIR}/assets/windows/vlc"
    )
    FIND_LIBRARY(
            LIBVLCCORE_LIBRARY
            NAMES vlccore libvlccore
            HINTS "$ENV{LIBVLC_LIBRARY_PATH}"
            PATHS
            "${PROJECT_SOURCE_DIR}/assets/linux/vlc"
            "${PROJECT_SOURCE_DIR}/assets/linux-arm/vlc"
            "${PROJECT_SOURCE_DIR}/assets/maxosx/vlc"
            "${PROJECT_SOURCE_DIR}/assets/windows/vlc"
    )
endif ()

IF (LIBVLC_INCLUDE_DIR AND LIBVLC_LIBRARY AND LIBVLCCORE_LIBRARY)
    SET(LIBVLC_FOUND TRUE)
ENDIF (LIBVLC_INCLUDE_DIR AND LIBVLC_LIBRARY AND LIBVLCCORE_LIBRARY)

if (LIBVLC_FOUND)
    if (NOT LIBVLC_FIND_QUIETLY)
        message(STATUS "Found LibVLC include-dir path: ${LIBVLC_INCLUDE_DIR}")
        message(STATUS "Found LibVLC library path:${LIBVLC_LIBRARY}")
        message(STATUS "Found LibVLCcore library path:${LIBVLCCORE_LIBRARY}")
    endif (NOT LIBVLC_FIND_QUIETLY)
else (LIBVLC_FOUND)
    if (LIBVLC_FIND_REQUIRED)
        message(STATUS "ENVIRONMENT LIBVLC_INCLUDE_PATH=$ENV{LIBVLC_INCLUDE_PATH}")
        message(STATUS "ENVIRONMENT LIBVLC_LIBRARY_PATH=$ENV{LIBVLC_LIBRARY_PATH}")
        message(STATUS "LibVLC include-dir path: ${LIBVLC_INCLUDE_DIR}")
        message(STATUS "LibVLC library path:${LIBVLC_LIBRARY}")
        message(STATUS "LibVLCcore library path:${LIBVLCCORE_LIBRARY}")
        message(FATAL_ERROR "Could not find LibVLC")
    endif (LIBVLC_FIND_REQUIRED)
endif (LIBVLC_FOUND)
