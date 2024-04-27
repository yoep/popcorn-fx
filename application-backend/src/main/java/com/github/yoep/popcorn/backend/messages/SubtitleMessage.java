package com.github.yoep.popcorn.backend.messages;

import com.github.yoep.popcorn.backend.utils.Message;
import lombok.Getter;

@Getter
public enum SubtitleMessage implements Message {
    SRT_DESCRIPTION("subtitle_srt_description"),
    ZIP_DESCRIPTION("subtitle_zip_description"),
    NONE("subtitle_none"),
    CUSTOM("subtitle_custom"),
    CHANGE("subtitles_change"),
    INCREASE_SUBTITLE_OFFSET("subtitles_increase_offset"),
    DECREASE_SUBTITLE_OFFSET("subtitles_decrease_offset"),
    INCREASE_FONT_SIZE("subtitles_increase_font_size"),
    DECREASE_FONT_SIZE("subtitles_decrease_font_size");

    private final String key;

    SubtitleMessage(String key) {
        this.key = key;
    }
}
