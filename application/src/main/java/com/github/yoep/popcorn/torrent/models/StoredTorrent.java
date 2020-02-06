package com.github.yoep.popcorn.torrent.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class StoredTorrent {
    private String name;
    private String magnetUri;
}
