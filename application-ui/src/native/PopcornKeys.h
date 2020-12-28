#ifndef POPCORNTIME_POPCORNKEYS_H
#define POPCORNTIME_POPCORNKEYS_H

#include "../../../shared/Log.h"
#include "IInputEventsBridge.h"

class PopcornKeys {
public:
    PopcornKeys(int argc, char **argv);

    ~PopcornKeys();

private:
    int _argc;
    char **_argv;
    IInputEventsBridge *_eventsBridge;
    Log *_log;

    void init();

    void parseArguments();
};

#endif //POPCORNTIME_POPCORNKEYS_H
