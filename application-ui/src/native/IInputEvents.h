#ifndef POPCORNTIME_IINPUTEVENTS_H
#define POPCORNTIME_IINPUTEVENTS_H

#include "MediaKeyType.h"

#include <functional>

/**
 * The input events interface which defined the abstract functions an input event handler should implement.
 */
class IInputEvents {
public:
    ~IInputEvents() = default;

    /**
     * Invoked when a media key is being pressed.
     *
     * @param mediaKeyPressed The media key press function.
     */
    virtual void onMediaKeyPressed(std::function<void(MediaKeyType)> mediaKeyPressed) = 0;
};

#endif //POPCORNTIME_IINPUTEVENTS_H
