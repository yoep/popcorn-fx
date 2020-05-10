package com.github.yoep.popcorn.activities;

import com.frostwire.jlibtorrent.TorrentInfo;

public interface ShowTorrentDetailsActivity extends ShowDetailsActivity {
    /**
     * Get the magnet uri that was used to resolve the torrent details.
     *
     * @return Returns the torrent magnet uri.
     */
    String getMagnetUri();

    /**
     * Get the torrent info that needs to be shown.
     *
     * @return Returns the loaded torrent information.
     */
    TorrentInfo getTorrentInfo();
}
