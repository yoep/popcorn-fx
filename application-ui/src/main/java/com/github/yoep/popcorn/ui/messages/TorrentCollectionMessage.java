package com.github.yoep.popcorn.ui.messages;

import com.github.yoep.popcorn.backend.utils.Message;
import lombok.Getter;

@Getter
public enum TorrentCollectionMessage implements Message {
    EMPTY("torrent_collection_empty"),
    LOADING("torrent_collection_loading");

    private final String key;

    TorrentCollectionMessage(String key) {
        this.key = key;
    }
}
