package com.github.yoep.popcorn.providers.media.models;

import lombok.Data;

@Data
public class Torrent {
    private String provider;
    private String filesize;
    private String size;
    private String peer;
    private String seed;
    private String url;
}
