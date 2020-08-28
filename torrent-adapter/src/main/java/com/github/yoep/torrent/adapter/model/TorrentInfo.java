package com.github.yoep.torrent.adapter.model;

import java.util.List;

public interface TorrentInfo {
    /**
     * Get the name of the torrent info.
     *
     * @return Returns the torrent info name.
     */
    String getName();

    /**
     * Get the total number of files which are present in the torrent.
     *
     * @return Returns the total number of files.
     */
    int getTotalFiles();

    /**
     * Get a list of torrent files contained in this torrent info.
     *
     * @return Returns a list of torrent info files.
     */
    List<TorrentFileInfo> getFiles();

    /**
     * Get the largest torrent file contained within this torrent info collection.
     *
     * @return Returns the largest torrent file info from this torrent info.
     */
    TorrentFileInfo getLargestFile();
}