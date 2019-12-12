package com.github.yoep.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum TorrentMessage implements Message {
    CONNECTING("torrent_connecting");

    private final String key;

    TorrentMessage(String key) {
        this.key = key;
    }
}
