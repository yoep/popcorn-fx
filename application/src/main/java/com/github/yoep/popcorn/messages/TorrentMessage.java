package com.github.yoep.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum TorrentMessage implements Message {
    INITIALIZING("torrent_initializing"),
    CONNECTING("torrent_connecting"),
    STARTING("torrent_starting"),
    DOWNLOADING("torrent_downloading"),
    DOWNLOADING_SUBTITLES("torrent_downloading_subtitles"),
    READY("torrent_ready"),
    FAILED("torrent_failed"),
    STORE_COLLECTION("torrent_store_collection"),
    REMOVE_COLLECTION("torrent_remove_collection");

    private final String key;

    TorrentMessage(String key) {
        this.key = key;
    }
}
