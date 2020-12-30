#ifndef POPCORNTIME_POPCORNKEYSEVENTMANAGER_H
#define POPCORNTIME_POPCORNKEYSEVENTMANAGER_H

#include "../../../shared/Log.h"
#include "MediaKeyType.h"
#include "PopcornKeysCallbacks.h"

#include <list>

class PopcornKeysEventManager {

public:
    PopcornKeysEventManager();

    /**
     * Add the given callback to the event manager.
     * When a media key is pressed, the given callback will be triggered by this event manager.
     *
     * @param callback The callback to register within the event manager.
     */
    void addMediaCallback(popcorn_keys_media_key_pressed_t callback);

    /**
     * Invoked when a media key has been pressed.
     *
     * @param mediaKeyType The media key type that has been pressed.
     */
    void onMediaKeyPressed(MediaKeyType mediaKeyType);

private:
    std::list<popcorn_keys_media_key_pressed_t> _mediaKeyCallbacks;
    Log *_log;
};

#endif //POPCORNTIME_POPCORNKEYSEVENTMANAGER_H
