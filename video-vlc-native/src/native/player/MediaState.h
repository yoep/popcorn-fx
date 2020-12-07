#ifndef POPCORNPLAYER_MEDIASTATE_H
#define POPCORNPLAYER_MEDIASTATE_H

/**
 * Enumeration of the possible states for a media item.
 */
enum class MediaState {
    UNKNOWN = -1,
    PARSING = 2,
    PARSED = 3,
    OPENING = 4,
    PLAYING = 0,
    PAUSED = 1,
    ENDED = 5,
    ERROR = 6
};

/**
 * Convert the given media state to a string.
 *
 * @param state The state to convert.
 * @return Returns the readable string for the given state.
 */
const char *media_state_as_string(MediaState state);

#endif //POPCORNPLAYER_MEDIASTATE_H
