#ifndef POPCORNTIME_POPCORNKEYS_H
#define POPCORNTIME_POPCORNKEYS_H

#include "../../../shared/Log.h"
#include "IInputEventsBridge.h"
#include "PopcornKeysEventManager.h"

class PopcornKeys {
public:
    PopcornKeys(int argc, char **argv);

    ~PopcornKeys();

    /**
     * Register the given media key pressed callback function.
     * The callback will be invoked when a media key has been pressed.
     *
     * @param callback The callback to invoke.
     */
    void addOnMediaKeyPressedCallback(popcorn_keys_media_key_pressed_t callback);

private:
    int _argc;
    char **_argv;
    IInputEventsBridge *_eventsBridge;
    PopcornKeysEventManager *_eventManager;
    Log *_log;

    void init();

    void parseArguments();
};

#endif //POPCORNTIME_POPCORNKEYS_H
