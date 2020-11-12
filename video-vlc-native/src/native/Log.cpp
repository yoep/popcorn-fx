#include "Log.h"
#include "AppProperties.h"
#include <iostream>
#include <sstream>

using namespace std;

Log* Log::instance = nullptr;

Log::Log() = default;

Log::~Log() = default;

Log* Log::getInstance()
{
    if (!instance) {
        instance = new Log;
    }

    return instance;
}

void Log::trace(const char* message)
{
    log(message, "TRACE");
}

void Log::debug(const char* message)
{
    log(message, "DEBUG");
}

void Log::info(const std::basic_string<char>& message)
{
    info(message.c_str());
}

void Log::info(const char* message)
{
    log(message, "INFO");
}

void Log::warn(const char* message)
{
    logToSysError(message, "WARN");
}

void Log::warn(const basic_string<char>& message)
{
    warn(message.c_str());
}

void Log::error(const basic_string<char>& message)
{
    error(message.c_str());
}

void Log::error(const basic_string<char>& message, const exception& ex)
{
    error(message.c_str(), ex);
}

void Log::error(const char* message)
{
    logToSysError(message, "ERROR");
}

void Log::error(const char* message, const std::exception& ex)
{
    error(message + std::string(", error: ") + ex.what());
}

void Log::log(const char* message, const char level[6]) { cout << appName() << " " << level << " - " << message << endl; }

void Log::logToSysError(const char* message, const char level[6])
{
    cerr << appName() << " " << level << " - " << message << endl;
}

basic_string<char> Log::appName()
{
    std::ostringstream oss;
    oss << "[" << ApplicationTitle << "]";
    std::string name = oss.str();

    return name;
}
