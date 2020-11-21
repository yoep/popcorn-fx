#ifndef POPCORNPLAYER_LOG_H
#define POPCORNPLAYER_LOG_H

#include <string>

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

class Log {
public:
    ~Log();

    static Log *getInstance();

    LogLevel getLevel();

    void setLevel(LogLevel level);

    void trace(const char *message);

    void debug(const char *message);

    void debug(const std::basic_string<char> &message);

    void info(const char *message);

    void info(const std::basic_string<char> &message);

    void warn(const char *message);

    void warn(const std::basic_string<char> &message);

    void error(const char *message);

    void error(const std::basic_string<char> &message);

    void error(const char *message, const std::exception &ex);

    void error(const std::basic_string<char> &message, const std::exception &ex);

private:
    Log();
    LogLevel level;

    static void log(const char *message, const char string[6]);

    static void logToSysError(const char *message, const char string[6]);

    static Log *instance;

    static std::__cxx11::basic_string<char> appName();
};

#endif //POPCORNPLAYER_LOG_H
