package com.github.yoep.player.chromecast.api.v2;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import su.litvak.chromecast.api.v2.Request;

import java.util.Collection;

@Data
@Builder
@AllArgsConstructor
public class Load implements Request {
    @JsonProperty
    private final String type = "LOAD";
    @JsonProperty
    private Long requestId;
    @JsonProperty
    private final String sessionId;
    @JsonProperty
    private final Media media;
    @JsonProperty
    private final boolean autoplay;
    @JsonProperty
    private final double currentTime;
    @JsonProperty
    private final Object customData;
    @JsonProperty
    private final Object queueData;
    @JsonProperty
    private final Collection<Integer> activeTrackIds;
}
