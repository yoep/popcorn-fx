package com.github.yoep.torrent.adapter.model;

import java.io.Serializable;
import java.util.List;
import java.util.Optional;

public interface TorrentInfo extends Serializable {
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

    /**
     * Get the torrent file based on the filename of the torrent file.
     *
     * @param filename The filename of the torrent file to search for.
     * @return Returns the torrent file if found, else {@link Optional#empty()}.
     */
    Optional<TorrentFileInfo> getByFilename(String filename);
}
