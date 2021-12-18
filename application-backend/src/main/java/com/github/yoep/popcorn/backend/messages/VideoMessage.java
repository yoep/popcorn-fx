package com.github.yoep.popcorn.backend.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum VideoMessage implements Message {
    SUBTITLES_OFFSET("video_subtitle_offset"),
    VIDEO_ERROR("video_unexpected_error"),
    SUBTITLE_DOWNLOAD_FILED("video_subtitle_failed");

    private final String key;

    VideoMessage(String key) {
        this.key = key;
    }
}
