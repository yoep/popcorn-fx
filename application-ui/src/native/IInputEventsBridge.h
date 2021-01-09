#ifndef POPCORNTIME_IINPUTEVENTSBRIDGE_H
#define POPCORNTIME_IINPUTEVENTSBRIDGE_H

#include "MediaKeyType.h"

#include <functional>

/**
 * The input events bridge interface.
 * The bridge interface is used to abstract the OS specific input event handlers.
 */
class IInputEventsBridge {
public:
    ~IInputEventsBridge() = default;

    /**
     * Register a media callback function which should be invoked when a media
     * key has been pressed.
     *
     * @param callback The function callback to invoked.
     */
    virtual void addMediaCallback(std::function<void(MediaKeyType)> callback) = 0;

    /**
     * Grab the media keys from the system event bridge.
     */
    virtual void grabMediaKeys() = 0;

    /**
     * Release the media keys.
     */
    virtual void releaseMediaKeys() = 0;
};

#endif //POPCORNTIME_IINPUTEVENTSBRIDGE_H
