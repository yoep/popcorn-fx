
#include "MediaKeyType.h"

const char *media_key_type_as_string(MediaKeyType type)
{
    switch (type) {
    case MediaKeyType::PLAY:
        return "PLAY";
    case MediaKeyType::PAUSE:
        return "PAUSE";
    case MediaKeyType::STOP:
        return "STOP";
    case MediaKeyType::PREVIOUS:
        return "PREVIOUS";
    case MediaKeyType::NEXT:
        return "NEXT";
    default:
        return "UNKNOWN";
    }
}
