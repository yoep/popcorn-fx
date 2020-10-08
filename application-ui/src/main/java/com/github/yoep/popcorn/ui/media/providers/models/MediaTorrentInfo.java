package com.github.yoep.popcorn.ui.media.providers.models;

import com.github.yoep.torrent.adapter.model.Torrent;
import lombok.Data;

import java.io.Serializable;

/**
 * Information about a torrent that can be used to instantiate a {@link Torrent}.
 * This model is primarily used for displaying media details.
 */
@Data
public class MediaTorrentInfo implements Serializable {
    private String provider;
    private String filesize;
    private long size;
    private int peer;
    private int seed;
    private String url;
}
