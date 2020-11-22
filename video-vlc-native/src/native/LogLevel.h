#ifndef POPCORNPLAYER_LOGLEVEL_H
#define POPCORNPLAYER_LOGLEVEL_H

/**
 * Enumeration of the possible log levels which are supported by the Logger.
 */
enum LogLevel {
    // 0001 1111
    TRACE = 31,
    // 0000 1111
    DEBUG = 15,
    // 0000 0111
    INFO = 7,
    // 0000 0011
    WARN = 3,
    // 0000 0001
    ERROR = 1
};

#endif //POPCORNPLAYER_LOGLEVEL_H
