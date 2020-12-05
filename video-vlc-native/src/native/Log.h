#ifndef POPCORNPLAYER_LOG_H
#define POPCORNPLAYER_LOG_H

#include "LogLevel.h"

#include <atomic>
#include <memory>
#include <string>

using namespace std;

class Log {
public:
    /**
     * Initialize a new Log instance.
     * Don't use this constructor directly, but use #getInstance() instead.
     */
    Log();

    ~Log();

    /**
     * Get the Log instance.
     *
     * @return Returns the Log instance.
     */
    static Log *instance();

    /**
     * Get the current log level of the logger.
     *
     * @return Returns the log level.
     */
    logLevel::LogLevel level();

    /**
     * Set the log level of the logger.
     *
     * @param level The new log level for the logger.
     */
    void setLevel(logLevel::LogLevel level);

    /**
     * Log a trace message with the logger.
     *
     * @param message The message to log.
     */
    void trace(const char *message);

    /**
     * Log a trace message with the logger.
     *
     * @param message The message to log.
     */
    void trace(const std::basic_string<char> &message);

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
    std::atomic<logLevel::LogLevel> _level = logLevel::INFO;
    static shared_ptr<Log> _instance;

    static void log(const char *message, const char string[6]);

    static void logToSysError(const char *message, const char string[6]);

    static std::__cxx11::basic_string<char> appName();
};

#endif //POPCORNPLAYER_LOG_H
