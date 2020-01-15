package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;

import java.util.Optional;

public interface LoadMediaTorrentActivity extends LoadTorrentActivity {
    /**
     * Get the selected torrent that needs to be preloaded.
     *
     * @return Returns the torrent that needs to be loaded.
     */
    MediaTorrentInfo getTorrent();

    /**
     * Get the media for which the torrent is being loaded.
     *
     * @return Returns the media of the torrent.
     */
    Media getMedia();

    /**
     * The quality of the torrent that should is being loaded.
     *
     * @return Returns the video quality of the torrent.
     */
    String getQuality();

    /**
     * Get the subtitle that needs to be loaded while loading the torrent.
     *
     * @return Returns the subtitle for the torrent, else {@link Optional#empty()}.
     */
    Optional<SubtitleInfo> getSubtitle();
}
