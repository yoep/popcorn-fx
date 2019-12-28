package com.github.yoep.popcorn.activities;

import com.github.yoep.popcorn.media.providers.models.TorrentInfo;
import com.github.yoep.popcorn.subtitle.models.Subtitle;

import java.util.Optional;

public interface LoadTorrentActivity extends PlayMediaActivity {
    /**
     * Get the torrent quality that should be played.
     *
     * @return Returns the torrent quality.
     */
    String getQuality();

    /**
     * Get the selected torrent that needs to be preloaded.
     *
     * @return Returns the torrent that needs to be loaded.
     */
    TorrentInfo getTorrent();

    /**
     * Get the subtitle that needs to be preloaded for the playback.
     *
     * @return Returns the subtitle for the playback if present, else {@link Optional#empty()}.
     */
    Optional<Subtitle> getSubtitle();
}
