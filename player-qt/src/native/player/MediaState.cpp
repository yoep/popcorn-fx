
#include "MediaState.h"

const char *media_state_as_string(MediaState state)
{
    switch (state) {
    case MediaState::UNKNOWN:
        return "UNKNOWN";
    case MediaState::OPENING:
        return "OPENING";
    case MediaState::PARSING:
        return "PARSING";
    case MediaState::PARSED:
        return "PARSED";
    case MediaState::PLAYING:
        return "PLAYING";
    case MediaState::PAUSED:
        return "PAUSED";
    case MediaState::ENDED:
        return "ENDED";
    case MediaState::ERROR:
        return "ERROR";
    default:
        return "NOT_MAPPED";
    }
}
