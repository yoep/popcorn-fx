package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum SubtitleMessage implements Message {
    INCREASE_SUBTITLE_OFFSET("subtitles_increase_offset"),
    DECREASE_SUBTITLE_OFFSET("subtitles_decrease_offset");

    private final String key;

    SubtitleMessage(String key) {
        this.key = key;
    }
}
