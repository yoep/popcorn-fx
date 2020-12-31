#ifndef POPCORNTIME_MEDIAKEYTYPE_H
#define POPCORNTIME_MEDIAKEYTYPE_H

enum class MediaKeyType {
    UNKNOWN = -1,
    STOP = 0,
    PLAY = 1,
    PAUSE = 2,
    PREVIOUS = 3,
    NEXT = 4,
    VOLUME_LOWER = 5,
    VOLUME_HIGHER = 6
};

/**
 * Convert the given media key type to a string.
 *
 * @param type The key media type to convert.
 * @return Returns the readable string for the given media key type.
 */
const char *media_key_type_as_string(MediaKeyType type);

#endif //POPCORNTIME_MEDIAKEYTYPE_H
