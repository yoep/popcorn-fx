package com.github.yoep.popcorn.activities;

import com.frostwire.jlibtorrent.TorrentInfo;

public interface LoadUrlTorrentActivity extends LoadTorrentActivity {
    /**
     * Get the torrent to load.
     *
     * @return Returns the torrent to load.
     */
    TorrentInfo getTorrentInfo();

    /**
     * Get the filename in the torrent that is being loaded.
     *
     * @return Returns the filename.
     */
    String getFilename();

    /**
     * Get the file index of the torrent to load.
     *
     * @return Returns the index of the file to load.
     */
    int getFileIndex();
}
