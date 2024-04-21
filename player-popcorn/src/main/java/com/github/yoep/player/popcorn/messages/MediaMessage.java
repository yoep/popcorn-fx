package com.github.yoep.player.popcorn.messages;

import com.github.yoep.popcorn.backend.utils.Message;
import lombok.Getter;

@Getter
public enum MediaMessage implements Message {
    SUBTITLE_NONE("media_subtitle_none");

    private final String key;

    MediaMessage(String key) {
        this.key = key;
    }
}
