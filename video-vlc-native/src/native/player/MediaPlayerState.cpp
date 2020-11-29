
#include "MediaPlayerState.h"

const char *media_player_state_as_string(MediaPlayerState state)
{
    switch (state) {
    case UNKNOWN:
        return "UNKNOWN";
    case PLAYING:
        return "PLAYING";
    case PAUSED:
        return "PAUSED";
    case BUFFERING:
        return "BUFFERING";
    case STOPPED:
        return "STOPPED";
    default:
        return "NOT_MAPPED";
    }
}
