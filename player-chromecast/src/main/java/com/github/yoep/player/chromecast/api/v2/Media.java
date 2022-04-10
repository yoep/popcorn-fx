package com.github.yoep.player.chromecast.api.v2;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;

import java.util.List;
import java.util.Map;

@Data
@Builder
@AllArgsConstructor
public class Media {
    @JsonProperty
    @JsonInclude(JsonInclude.Include.NON_NULL)
    public final Map<String, Object> metadata;

    @JsonProperty("contentId")
    public final String url;

    @JsonProperty
    @JsonInclude(JsonInclude.Include.NON_NULL)
    public final Double duration;

    @JsonProperty
    @JsonInclude(JsonInclude.Include.NON_NULL)
    public final su.litvak.chromecast.api.v2.Media.StreamType streamType;

    @JsonProperty
    public final String contentType;

    @JsonProperty
    @JsonInclude(JsonInclude.Include.NON_NULL)
    public final Map<String, Object> customData;

    @JsonProperty
    public final Map<String, Object> textTrackStyle;

    @JsonProperty
    public final List<Track> tracks;
}
