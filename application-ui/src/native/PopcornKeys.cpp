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
    this->_eventManager = new PopcornKeysEventManager();

    init();
}

PopcornKeys::~PopcornKeys()
{
    _log->debug("Releasing the Popcorn Keys resources");
    delete _eventsBridge;
    delete _eventManager;
}

void PopcornKeys::addOnMediaKeyPressedCallback(popcorn_keys_media_key_pressed_t callback)
{
    // pass the registration to the event manager
    _eventManager->addMediaCallback(callback);
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

    // add the media key callback function to the bridge
    _eventsBridge->addMediaCallback([&](MediaKeyType type) {
        _eventManager->onMediaKeyPressed(type);
    });

    this->_log->debug("Popcorn keys has been initialized");
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
