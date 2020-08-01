package com.github.yoep.popcorn.ui.torrent.models;

import lombok.Data;

import java.util.ArrayList;
import java.util.List;

@Data
public class TorrentCollection {
    private List<StoredTorrent> torrents = new ArrayList<>();
}
