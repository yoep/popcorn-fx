package com.github.yoep.popcorn.backend.adapters.torrent.model;

import java.io.Serializable;
import java.util.List;
import java.util.Optional;

/**
 * The {@link TorrentInfo} interface represents information about a torrent, including its magnet URI, name, directory name,
 * total number of files, list of files, and methods to retrieve specific file information.
 */
public interface TorrentInfo extends Serializable {
    /**
     * Get the magnet URI of the torrent.
     *
     * @return The magnet URI of the torrent.
     */
    String getMagnetUri();

    /**
     * Get the name of the torrent.
     *
     * @return The name of the torrent.
     */
    String getName();

    /**
     * Get the directory name of the torrent.
     *
     * @return The directory name of the torrent.
     */
    String getDirectoryName();

    /**
     * Get the total number of files present in the torrent.
     *
     * @return The total number of files in the torrent.
     */
    int getTotalFiles();

    /**
     * Get a list of files contained in this torrent.
     *
     * @return A list of torrent files.
     */
    List<TorrentFileInfo> getFiles();

    /**
     * Get the largest torrent file contained within this torrent.
     *
     * @return The largest torrent file information.
     */
    TorrentFileInfo getLargestFile();

    /**
     * Get the torrent file based on its filename.
     *
     * @param filename The filename of the torrent file to search for.
     * @return An {@link Optional} containing the torrent file if found, otherwise {@link Optional#empty()}.
     */
    Optional<TorrentFileInfo> getByFilename(String filename);
}
