#include "Log.h"

#include "AppProperties.h"
#include "LogLevelFlags.h"

#include <iostream>
#include <sstream>

using namespace std;

shared_ptr<Log> Log::_instance = nullptr;

Log::Log() = default;

Log::~Log() = default;

Log *Log::instance()
{
    if (!_instance) {
        _instance = std::make_shared<Log>();
    }

    return _instance.get();
}

logLevel::LogLevel Log::level()
{
    return this->_level;
}

void Log::setLevel(logLevel::LogLevel logLevel)
{
    this->_level = logLevel;
}

void Log::trace(const char *message)
{
    if (_level & TRACE_FLAG) {
        log(message, "TRACE");
    }
}

void Log::trace(const basic_string<char> &message)
{
    if (_level & TRACE_FLAG) {
        trace(message.c_str());
    }
}

void Log::debug(const char *message)
{
    if (_level & DEBUG_FLAG) {
        log(message, "DEBUG");
    }
}

void Log::debug(const basic_string<char> &message)
{
    if (_level & DEBUG_FLAG) {
        debug(message.c_str());
    }
}

void Log::info(const std::basic_string<char> &message)
{
    if (_level & INFO_FLAG) {
        info(message.c_str());
    }
}

void Log::info(const char *message)
{
    if (_level & INFO_FLAG) {
        log(message, "INFO");
    }
}

void Log::warn(const char *message)
{
    if (_level & WARN_FLAG) {
        logToSysError(message, "WARN");
    }
}

void Log::warn(const basic_string<char> &message)
{
    if (_level & WARN_FLAG) {
        warn(message.c_str());
    }
}

void Log::error(const basic_string<char> &message)
{
    if (_level & ERROR_FLAG) {
        error(message.c_str());
    }
}

void Log::error(const basic_string<char> &message, const exception &ex)
{
    if (_level & ERROR_FLAG) {
        error(message.c_str(), ex);
    }
}

void Log::error(const char *message)
{
    if (_level & ERROR_FLAG) {
        logToSysError(message, "ERROR");
    }
}

void Log::error(const char *message, const std::exception &ex)
{
    if (_level & ERROR_FLAG) {
        error(message + std::string(", error: ") + ex.what());
    }
}

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
