package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.io.Serializable;
import java.util.Optional;

/**
 * Information about a torrent that can be used to instantiate a {@link Torrent}.
 * This model is primarily used for displaying media details.
 */
@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"url","provider","source","title","quality","seed","peer","size","filesize", "file"})
public class MediaTorrentInfo extends Structure implements Serializable, Closeable {
    public String url;
    public String provider;
    public String source;
    public String title;
    public String quality;
    public int seed;
    public int peer;
    public String size;
    public String filesize;
    public String file;

    //region Getters & Setters

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

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
