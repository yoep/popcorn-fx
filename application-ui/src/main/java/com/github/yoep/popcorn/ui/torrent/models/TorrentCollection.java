package com.github.yoep.popcorn.ui.torrent.models;

import com.github.yoep.popcorn.backend.torrent.collection.StoredTorrent;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.ArrayList;
import java.util.List;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class TorrentCollection {
    private List<StoredTorrent> torrents = new ArrayList<>();
}
