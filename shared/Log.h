#ifndef POPCORN_SHARED_LOG_H
#define POPCORN_SHARED_LOG_H

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
     * Parse the log level from the command line arguments.
     * The arguments are matched against the "-l" option from which the value is retrieved and stored in the result.
     *
     * @param argc The total arguments.
     * @param argv The arguments.
     * @param result The result pointer in which the parsed log level will be stored.
     * @see logLevel::LogLevel
     */
    static void parseLogLevel(int argc, char **argv, logLevel::LogLevel *result);

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
     * Set the application name to use within the logger.
     *
     * @param name The name of the application to log.
     */
    void setApplicationName(const char *name);

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
    std::atomic<const char *> _appName;
    static shared_ptr<Log> _instance;

    void log(const char *message, const char string[6]);

    void logToSysError(const char *message, const char string[6]);

    std::__cxx11::basic_string<char> appName();
};

#endif //POPCORN_SHARED_LOG_H
