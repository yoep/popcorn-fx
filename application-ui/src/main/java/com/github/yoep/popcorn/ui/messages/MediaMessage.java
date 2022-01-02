package com.github.yoep.popcorn.ui.messages;

import com.github.spring.boot.javafx.text.Message;
import lombok.Getter;

@Getter
public enum MediaMessage implements Message {
    SEASONS("media_seasons"),
    PLURAL("media_plural"),
    SUBTITLE_NONE("media_subtitle_none"),
    VIDEO_FAILED_TO_OPEN("media_video_failed_to_open"),
    VIDEO_DOES_NOT_EXIST("media_video_does_not_exist"),
    URL_FAILED_TO_PROCESS("media_failed_to_process_url"),
    VIDEO_PLAYBACK_FAILED("media_video_playback_failed");

    private final String key;

    MediaMessage(String key) {
        this.key = key;
    }
}
