package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;

public interface PlayerManagerListener {
    void activePlayerChanged(PlayerChanged playerChange);

    void playersChanged();

    /**
     * Invoked when the currently active player's request has changed.
     *
     * @param request The new play request of the player.
     */
    void onPlayerPlaybackChanged(PlayRequest request);

    /**
     * Invoked when the currently active player's time has changed.
     *
     * @param newTime The new time of the active player.
     */
    void onPlayerTimeChanged(Long newTime);

    /**
     * Invoked when the currently active player's duration has changed.
     *
     * @param newDuration The new duration of the active player.
     */
    void onPlayerDurationChanged(Long newDuration);

    /**
     * Invoked when the currently active player's state has changed.
     *
     * @param newState The new state of the active player.
     */
    void onPlayerStateChanged(Player.State newState);
}
