package com.github.yoep.popcorn.activities;

import com.frostwire.jlibtorrent.TorrentInfo;

public interface ShowTorrentDetailsActivity extends OverlayActivity {
    /**
     * Get the torrent info that needs to be shown.
     *
     * @return Returns the loaded torrent information.
     */
    TorrentInfo getTorrentInfo();
}
