package com.github.yoep.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum TorrentMessage implements Message {
    INITIALIZING("torrent_initializing"),
    CONNECTING("torrent_connecting"),
    DOWNLOADING("torrent_downloading");

    private final String key;

    TorrentMessage(String key) {
        this.key = key;
    }
}
