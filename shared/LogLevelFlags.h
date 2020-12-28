#ifndef POPCORN_SHARED_LOGLEVELFLAGS_H
#define POPCORN_SHARED_LOGLEVELFLAGS_H

/**
 * Enumeration of the log level flags used to determine the active logging state.
 */
enum LevelFlags {
    // 0001 0000
    TRACE_FLAG = 16,
    // 0000 1000
    DEBUG_FLAG = 8,
    // 0000 0100
    INFO_FLAG = 4,
    // 0000 0010
    WARN_FLAG = 2,
    // 0000 0001
    ERROR_FLAG = 1
};

#endif //POPCORN_SHARED_LOGLEVELFLAGS_H
