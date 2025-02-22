package com.github.yoep.popcorn.backend.adapters.torrent.model;

import java.io.Serializable;

public interface TorrentFileInfo extends Serializable, Comparable<TorrentFileInfo> {
    /**
     * Get the filename of the torrent file.
     *
     * @return Returns the filename.
     */
    String getFilename();

    /**
     * Get the file path of the torrent file within the torrent.
     * This path is relative to the torrent directory.
     *
     * @return Returns the file path of the torrent file.
     */
    String getFilePath();

    /**
     * Get the file size of the torrent file.
     *
     * @return Returns the size of the torrent in bytes.
     */
    Long getFileSize();

    /**
     * Get the index of the file in the torrent.
     *
     * @return Returns the file index.
     */
    int getFileIndex();
}
