FIND_PATH(LIBVLC_INCLUDE_DIR vlc/vlc.h
        HINTS "$ENV{LIBVLC_INCLUDE_PATH}"
        PATHS
        "${CMAKE_CURRENT_SOURCE_DIR}/libvlc"
        "${CMAKE_CURRENT_SOURCE_DIR}/libvlc/vlc"
        "$ENV{LIB_DIR}/include"
        "$ENV{LIB_DIR}/include/vlc"
        "/usr/include"
        "/usr/include/vlc"
        "/usr/local/include"
        "/usr/local/include/vlc"
        #mingw
        c:/msys/local/include
        )
FIND_PATH(LIBVLC_INCLUDE_DIR PATHS "${CMAKE_INCLUDE_PATH}/vlc" NAMES vlc.h)

find_library(LIBVLC_LIBRARY NAMES vlc libvlc
        PATHS
        "$ENV{LIBVLC_LIBRARY_PATH}"
        "$ENV{LIB_DIR}/lib"
        #Mac OS
        "${CMAKE_CURRENT_SOURCE_DIR}/contribs/lib"
        "${CMAKE_CURRENT_SOURCE_DIR}/contribs/plugins"
        "/Applications/VLC.app/Contents/MacOS/lib"
        "/Applications/VLC.app/Contents/MacOS/plugins"
        #mingw
        c:/msys/local/lib
        "c:/Program Files/VideoLAN/VLC/"
        )
FIND_LIBRARY(LIBVLC_LIBRARY NAMES vlc libvlc)
FIND_LIBRARY(LIBVLCCORE_LIBRARY NAMES vlccore libvlccore
        PATHS
        "$ENV{LIBVLC_LIBRARY_PATH}"
        "$ENV{LIB_DIR}/lib"
        #Mac OS
        "${CMAKE_CURRENT_SOURCE_DIR}/contribs/lib"
        "${CMAKE_CURRENT_SOURCE_DIR}/contribs/plugins"
        "/Applications/VLC.app/Contents/MacOS/lib"
        "/Applications/VLC.app/Contents/MacOS/plugins"
        #mingw
        c:/msys/local/lib
        "c:/Program Files/VideoLAN/VLC/"
        )
FIND_LIBRARY(LIBVLCCORE_LIBRARY NAMES vlccore libvlccore)

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
        message(STATUS "ENVIRONMENT LIB_DIR=$ENV{LIB_DIR}")
        message(STATUS "ENVIRONMENT LIBVLC_INCLUDE_PATH=$ENV{LIBVLC_INCLUDE_PATH}")
        message(STATUS "ENVIRONMENT LIBVLC_LIBRARY_PATH=$ENV{LIBVLC_LIBRARY_PATH}")
        message(STATUS "LibVLC include-dir path: ${LIBVLC_INCLUDE_DIR}")
        message(STATUS "LibVLC library path:${LIBVLC_LIBRARY}")
        message(STATUS "LibVLCcore library path:${LIBVLCCORE_LIBRARY}")
        message(FATAL_ERROR "Could not find LibVLC")
    endif (LIBVLC_FIND_REQUIRED)
endif (LIBVLC_FOUND)
