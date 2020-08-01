package com.github.yoep.popcorn.ui.activities;

import com.github.yoep.torrent.adapter.model.TorrentFileInfo;
import com.github.yoep.torrent.adapter.model.TorrentInfo;

public interface LoadUrlTorrentActivity extends LoadTorrentActivity {
    /**
     * Get the torrent to load.
     *
     * @return Returns the torrent to load.
     */
    TorrentInfo getTorrentInfo();

    /**
     * Get the torrent file info that has been selected.
     *
     * @return Returns the torrent file info.
     */
    TorrentFileInfo getTorrentFileInfo();
}
