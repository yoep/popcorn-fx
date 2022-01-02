
#include "MediaPlayerState.h"

const char *media_player_state_as_string(MediaPlayerState state)
{
    switch (state) {
    case MediaPlayerState::UNKNOWN:
        return "UNKNOWN";
    case MediaPlayerState::PLAYING:
        return "PLAYING";
    case MediaPlayerState::PAUSED:
        return "PAUSED";
    case MediaPlayerState::BUFFERING:
        return "BUFFERING";
    case MediaPlayerState::STOPPED:
        return "STOPPED";
    default:
        return "NOT_MAPPED";
    }
}
