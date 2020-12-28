#include "Log.h"

#include "LogLevelFlags.h"

#include <cstring>
#include <iostream>
#include <regex>
#include <vector>

using namespace std;

shared_ptr<Log> Log::_instance = nullptr;

Log::Log()
{
    this->_appName = {};
}

Log::~Log() = default;

Log *Log::instance()
{
    if (!_instance) {
        _instance = std::make_shared<Log>();
    }

    return _instance.get();
}

void Log::parseLogLevel(int argc, char **argv, logLevel::LogLevel *result)
{
    std::regex levelRegex("^(-l)(\\s|=)?([a-zA-Z]*)");
    char *levelArg = nullptr;

    for (int i = 0; i < argc; ++i) {
        auto argument = argv[i];
        std::cmatch matches;

        // check if the current argument matched
        if (std::regex_search(argument, matches, levelRegex)) {
            if (matches.size() == 4) {
                auto &match = matches[3];
                auto value = match.str();

                if (value.empty()) {
                    // assume that the level value is the next within the array
                    levelArg = argv[i + 1];
                } else {
                    levelArg = value.data();
                }
            }

            break;
        }
    }

    // check if a level arg was found
    // if not, exit the parsing
    if (levelArg == nullptr) {
        return;
    }

    // put the level to lower case
    for (int i = 0; i < strlen(levelArg); i++) {
        levelArg[i] = std::tolower(levelArg[i]);
    }

    std::string levelString(levelArg);

    if (levelString == "trace") {
        *result = logLevel::TRACE;
    } else if (levelString == "debug") {
        *result = logLevel::DEBUG;
    } else if (levelString == "info") {
        *result = logLevel::INFO;
    } else if (levelString == "warn") {
        *result = logLevel::WARN;
    } else if (levelString == "error") {
        *result = logLevel::ERROR;
    }
}

logLevel::LogLevel Log::level()
{
    return this->_level;
}

void Log::setLevel(logLevel::LogLevel logLevel)
{
    this->_level = logLevel;
}

void Log::setApplicationName(const char *name)
{
    this->_appName = name;
}

void Log::trace(const char *message)
{
    if (_level & TRACE_FLAG) {
        log(message, "TRACE");
    }
}

void Log::trace(const basic_string<char> &message)
{
    if (this->_level & TRACE_FLAG) {
        trace(message.c_str());
    }
}

void Log::debug(const char *message)
{
    if (this->_level & DEBUG_FLAG) {
        log(message, "DEBUG");
    }
}

void Log::debug(const basic_string<char> &message)
{
    if (this->_level & DEBUG_FLAG) {
        debug(message.c_str());
    }
}

void Log::info(const std::basic_string<char> &message)
{
    if (this->_level & INFO_FLAG) {
        info(message.c_str());
    }
}

void Log::info(const char *message)
{
    if (this->_level & INFO_FLAG) {
        log(message, "INFO");
    }
}

void Log::warn(const char *message)
{
    if (this->_level & WARN_FLAG) {
        logToSysError(message, "WARN");
    }
}

void Log::warn(const basic_string<char> &message)
{
    if (this->_level & WARN_FLAG) {
        warn(message.c_str());
    }
}

void Log::error(const basic_string<char> &message)
{
    if (this->_level & ERROR_FLAG) {
        error(message.c_str());
    }
}

void Log::error(const basic_string<char> &message, const exception &ex)
{
    if (this->_level & ERROR_FLAG) {
        error(message.c_str(), ex);
    }
}

void Log::error(const char *message)
{
    if (this->_level & ERROR_FLAG) {
        logToSysError(message, "ERROR");
    }
}

void Log::error(const char *message, const std::exception &ex)
{
    if (this->_level & ERROR_FLAG) {
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
    oss << "[" << this->_appName << "]";
    std::string name = oss.str();

    return name;
}
