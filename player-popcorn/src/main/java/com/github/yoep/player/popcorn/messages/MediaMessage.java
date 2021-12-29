package com.github.yoep.player.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum MediaMessage implements Message {
    SUBTITLE_NONE("media_subtitle_none");

    private final String key;

    MediaMessage(String key) {
        this.key = key;
    }
}
