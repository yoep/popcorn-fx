package com.github.yoep.player.chromecast.api.v2;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

/**
 * https://developers.google.com/cast/docs/reference/web_sender/chrome.cast.media.Track
 */
@Getter
@ToString
@EqualsAndHashCode
public class Track {
    @JsonProperty
    private final long trackId;
    @JsonProperty
    private final su.litvak.chromecast.api.v2.Track.TrackType type;
    @JsonProperty
    private final String trackContentId;
    @JsonProperty
    private final String trackContentType;
    @JsonProperty
    private final TextTrackType subtype;
    @JsonProperty
    private final String language;
    @JsonProperty
    private final String name;

    @Builder
    public Track(long trackId, su.litvak.chromecast.api.v2.Track.TrackType type, String trackContentId, String trackContentType, TextTrackType subtype,
                 String language, String name) {
        this.trackId = trackId;
        this.type = type;
        this.trackContentId = trackContentId;
        this.trackContentType = trackContentType;
        this.subtype = subtype;
        this.language = language;
        this.name = name;
    }
}
