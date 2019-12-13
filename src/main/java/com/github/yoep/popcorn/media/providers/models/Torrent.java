package com.github.yoep.popcorn.media.providers.models;

import lombok.Data;

@Data
public class Torrent {
    private String provider;
    private String filesize;
    private long size;
    private int peer;
    private int seed;
    private String url;
}
