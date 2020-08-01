package com.github.yoep.popcorn.ui.media.providers.models;

import com.github.yoep.torrent.adapter.model.Torrent;
import lombok.Data;

/**
 * Information about a torrent that can be used to instantiate a {@link Torrent}.
 * This model is primarily used for displaying media details.
 */
@Data
public class MediaTorrentInfo {
    private String provider;
    private String filesize;
    private long size;
    private int peer;
    private int seed;
    private String url;
}
