#include "Log.h"

#include "AppProperties.h"

#include <iostream>
#include <sstream>

using namespace std;

Log *Log::instance = nullptr;

// region Constructors

Log::Log()
{
    this->level = INFO;
}

Log::~Log() = default;

//endregion

//region Methods

Log *Log::getInstance()
{
    if (!instance) {
        instance = new Log;
    }

    return instance;
}

LogLevel Log::getLevel()
{
    return this->level;
}

void Log::setLevel(LogLevel logLevel)
{
    this->level = logLevel;
}

void Log::trace(const char *message)
{
    if (level & TRACE_FLAG) {
        log(message, "TRACE");
    }
}

void Log::debug(const char *message)
{
    if (level & DEBUG_FLAG) {
        log(message, "DEBUG");
    }
}

void Log::debug(const basic_string<char> &message)
{
    if (level & DEBUG_FLAG) {
        debug(message.c_str());
    }
}

void Log::info(const std::basic_string<char> &message)
{
    if (level & INFO_FLAG) {
        info(message.c_str());
    }
}

void Log::info(const char *message)
{
    if (level & INFO_FLAG) {
        log(message, "INFO");
    }
}

void Log::warn(const char *message)
{
    if (level & WARN_FLAG) {
        logToSysError(message, "WARN");
    }
}

void Log::warn(const basic_string<char> &message)
{
    if (level & WARN_FLAG) {
        warn(message.c_str());
    }
}

void Log::error(const basic_string<char> &message)
{
    if (level & ERROR_FLAG) {
        error(message.c_str());
    }
}

void Log::error(const basic_string<char> &message, const exception &ex)
{
    if (level & ERROR_FLAG) {
        error(message.c_str(), ex);
    }
}

void Log::error(const char *message)
{
    if (level & ERROR_FLAG) {
        logToSysError(message, "ERROR");
    }
}

void Log::error(const char *message, const std::exception &ex)
{
    if (level & ERROR_FLAG) {
        error(message + std::string(", error: ") + ex.what());
    }
}

//endregion

//region Functions

void Log::log(const char *message, const char level[6]) { cout << appName() << " " << level << " - " << message << endl; }

void Log::logToSysError(const char *message, const char level[6])
{
    cerr << appName() << " " << level << " - " << message << endl;
}

basic_string<char> Log::appName()
{
    std::ostringstream oss;
    oss << "[" << APPLICATION_TITLE << "]";
    std::string name = oss.str();

    return name;
}

//endregion
