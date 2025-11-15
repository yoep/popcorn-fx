package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import lombok.Getter;

@Getter
public class SubtitleException extends RuntimeException {
    private final Subtitle.Error.Type errorType;

    public SubtitleException(Subtitle.Error.Type errorType, String message) {
        super(message);
        this.errorType = errorType;
    }
}
