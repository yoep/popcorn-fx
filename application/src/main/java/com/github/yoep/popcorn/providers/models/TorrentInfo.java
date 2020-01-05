package com.github.yoep.popcorn.providers.models;

import com.github.yoep.popcorn.torrent.models.Torrent;
import lombok.Data;

/**
 * Information about a torrent that can be used to instantiate a {@link Torrent}.
 * This model is primarily used for displaying media details.
 */
@Data
public class TorrentInfo {
    private String provider;
    private String filesize;
    private long size;
    private int peer;
    private int seed;
    private String url;
}
