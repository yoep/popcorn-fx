package com.github.yoep.player.popcorn.listeners;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;

/**
 * The playback listener listens to playback events from the player.
 */
public interface PlaybackListener {
    /**
     * Invoked when a new {@link Player.PlayRequest} is triggered for a new video playback.
     *
     * @param request The playback information.
     */
    void onPlay(Player.PlayRequest request);

    /**
     * Invoked when the playback is resumed.
     */
    void onResume();

    /**
     * Invoked when the playback is paused.
     */
    void onPause();

    /**
     * Invoked when a time is seeked within the current playback.
     *
     * @param time The new playback time.
     */
    void onSeek(long time);

    /**
     * Invoked when the volume is changed of the player.
     *
     * @param volume The volume level of the player (0 = mute, 100 = max).
     */
    void onVolume(int volume);

    /**
     * Invoked when the playback is being stopped.
     */
    void onStop();
}
