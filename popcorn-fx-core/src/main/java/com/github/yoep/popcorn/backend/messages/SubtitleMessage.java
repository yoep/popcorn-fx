package com.github.yoep.popcorn.backend.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum SubtitleMessage implements Message {
    SRT_DESCRIPTION("subtitle_srt_description"),
    ZIP_DESCRIPTION("subtitle_zip_description"),
    NONE("subtitle_none"),
    CUSTOM("subtitle_custom");

    private final String key;

    SubtitleMessage(String key) {
        this.key = key;
    }
}
