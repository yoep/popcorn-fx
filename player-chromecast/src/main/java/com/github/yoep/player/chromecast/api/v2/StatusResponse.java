package com.github.yoep.player.chromecast.api.v2;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.AllArgsConstructor;
import lombok.Data;
import su.litvak.chromecast.api.v2.Response;

@Data
@AllArgsConstructor
public class StatusResponse implements Response {
    private Long requestId;
    @JsonProperty
    private final su.litvak.chromecast.api.v2.Status status;
}
