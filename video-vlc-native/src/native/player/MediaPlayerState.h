#ifndef POPCORNPLAYER_MEDIAPLAYERSTATE_H
#define POPCORNPLAYER_MEDIAPLAYERSTATE_H

/**
 * Enumeration of the possible states of a media player.
 */
enum MediaPlayerState {
    UNKNOWN = -1,
    PLAYING = 1,
    PAUSED = 2,
    BUFFERING = 3,
    STOPPED = 0
};

/**
 * Convert the given player state to a string.
 *
 * @param state The state to convert.
 * @return Returns the readable string for the given state.
 */
const char *media_player_state_as_string(MediaPlayerState state);

#endif //POPCORNPLAYER_MEDIAPLAYERSTATE_H
