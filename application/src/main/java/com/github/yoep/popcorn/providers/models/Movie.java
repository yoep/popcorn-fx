package com.github.yoep.popcorn.providers.models;

import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.util.Map;

@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
@Data
public class Movie extends AbstractMedia {
    private String trailer;
    private Map<String, Map<String, TorrentInfo>> torrents;
}
