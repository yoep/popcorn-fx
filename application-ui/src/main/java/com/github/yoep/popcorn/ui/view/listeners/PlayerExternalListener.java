package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;

/**
 * The {@link PlayerExternalListener} interface defines callbacks for events related to external interactions with the {@link Player}.
 * Implementations of this interface can listen for changes in playback requests, playback time, duration, player state, and download status.
 */
public interface PlayerExternalListener {
    /**
     * Invoked when the current playback request is changed.
     *
     * @param request The new playback request.
     */
    void onRequestChanged(PlayRequest request);

    /**
     * Invoked when the current playback time is changed.
     *
     * @param time The new playback time in milliseconds.
     */
    void onTimeChanged(long time);

    /**
     * Invoked when the duration of the playback is changed.
     *
     * @param duration The new playback duration in milliseconds.
     */
    void onDurationChanged(long duration);

    /**
     * Invoked when the player state is changed.
     *
     * @param state The new player state.
     */
    void onStateChanged(Player.State state);

    /**
     * Invoked when the download status is changed.
     *
     * @param status The new download status.
     */
    void onDownloadStatus(DownloadStatus status);
}

