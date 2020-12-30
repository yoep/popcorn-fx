#include "PopcornKeysEventManager.h"

PopcornKeysEventManager::PopcornKeysEventManager()
{
    this->_log = Log::instance();

    this->_mediaKeyCallbacks = {};
}

void PopcornKeysEventManager::addMediaCallback(popcorn_keys_media_key_pressed_t callback)
{
    _mediaKeyCallbacks.push_back(callback);
}

void PopcornKeysEventManager::onMediaKeyPressed(MediaKeyType mediaKeyType)
{
    _log->debug(std::string("Received media key pressed: ") + media_key_type_as_string(mediaKeyType));
    for (auto callback : _mediaKeyCallbacks) {
        callback(static_cast<int>(mediaKeyType));
    }
}
