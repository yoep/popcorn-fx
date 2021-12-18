package com.github.yoep.popcorn.ui.media.providers.models;

import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import lombok.Data;

import java.io.Serializable;
import java.util.Optional;

/**
 * Information about a torrent that can be used to instantiate a {@link Torrent}.
 * This model is primarily used for displaying media details.
 */
@Data
public class MediaTorrentInfo implements Serializable {
    private String provider;
    private String filesize;
    private String file;
    private long size;
    private int peer;
    private int seed;
    private String url;

    //region Getters & Setters

    /**
     * Get the file size of the torrent.
     * This field is only present when the torrent contains one media file.
     *
     * <i>Note:</i>
     * If {@link #getFile()} is present, then this field will be {@link Optional#empty()}.
     *
     * @return Returns the file size of the torrent if present.
     */
    public Optional<String> getFilesize() {
        return Optional.ofNullable(filesize);
    }

    /**
     * Get the file name of the torrent to download.
     * This field is only present when multiple media files are present within the torrent.
     *
     * <i>Note:</i>
     * If {@link #getFilesize()} is present, then this field will be {@link Optional#empty()}.
     *
     * @return Returns the file to download of the torrent if present.
     */
    public Optional<String> getFile() {
        return Optional.ofNullable(file);
    }

    /**
     * Set the number of peers for the torrent.
     * This is an alternative to {@link #setPeer(int)}.
     *
     * @param peers The number of peers.
     */
    public void setPeers(int peers) {
        this.peer = peers;
    }

    /**
     * Set the number of seeds for the torrent.
     * This is an alternative to {@link #setSeed(int)}.
     *
     * @param seeds The number of seeds.
     */
    public void setSeeds(int seeds) {
        this.seed = seeds;
    }

    //endregion
}
