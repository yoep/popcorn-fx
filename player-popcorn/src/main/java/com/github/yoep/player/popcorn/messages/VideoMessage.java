package com.github.yoep.player.popcorn.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum VideoMessage implements Message {
    SUBTITLES_OFFSET("video_subtitle_offset"),
    SUBTITLE_DOWNLOAD_FILED("video_subtitle_failed"),
    VIDEO_ERROR("video_unexpected_error"),
    VIDEO_VOLUME("video_volume");

    private final String key;

    VideoMessage(String key) {
        this.key = key;
    }
}
