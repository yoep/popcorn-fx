package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import org.springframework.lang.Nullable;

public interface PlayerExternalListener {
    /**
     * Invoked when the playback title is changed.
     *
     * @param title The new playback title.
     */
    void onTitleChanged(String title);

    /**
     * Invoked when the media playback is changed.
     *
     * @param media The new media item.
     */
    void onMediaChanged(@Nullable Media media);

    /**
     * Invoked when the current playback time is changed.
     *
     * @param time The new playback time.
     */
    void onTimeChanged(long time);

    /**
     * Invoked when the duration of the playback is changed.
     *
     * @param duration The new playback duration.
     */
    void onDurationChanged(long duration);

    /**
     * Invoked when the player state is changed.
     *
     * @param state The new player state.
     */
    void onStateChanged(PlayerState state);

    /**
     * Invoked when the download status is changed.
     *
     * @param status The new download status.
     */
    void onDownloadStatus(DownloadStatus status);
}
