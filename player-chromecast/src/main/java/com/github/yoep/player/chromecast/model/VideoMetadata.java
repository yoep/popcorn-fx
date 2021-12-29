package com.github.yoep.player.chromecast.model;

import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

@Getter
@Builder
@ToString
@EqualsAndHashCode
public class VideoMetadata {
    public static long UNKNOWN_DURATION = -1L;

    private final String contentType;
    private final Long duration;
}
