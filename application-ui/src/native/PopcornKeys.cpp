#include "PopcornKeys.h"

#if defined(linux)
#include <linux/LinuxInputEventsBridge.h>
#endif

PopcornKeys::PopcornKeys(int argc, char **argv)
{
    this->_log = Log::instance();
    this->_log->setApplicationName("Popcorn Keys");

    this->_argc = argc;
    this->_argv = argv;
    this->_eventsBridge = nullptr;

    init();
}

PopcornKeys::~PopcornKeys()
{
    delete _log;
}

void PopcornKeys::init()
{
    // parse the command argument
    parseArguments();

    // initialize the events bridge based on the platform type
#if defined(linux)
    _eventsBridge = new LinuxInputEventsBridge();
#elif defined(_WIN32)

#endif

    this->_log->debug("initialized");
}

void PopcornKeys::parseArguments()
{
    auto *result = (logLevel::LogLevel *)malloc(sizeof(enum logLevel::LogLevel));
    Log::parseLogLevel(_argc, _argv, result);

    if (result != nullptr) {
        this->_log->setLevel(*result);
    }

    // free the allocated memory for the log level
    free(result);
}
