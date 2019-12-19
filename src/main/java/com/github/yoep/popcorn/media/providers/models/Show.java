package com.github.yoep.popcorn.media.providers.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
@Data
public class Show extends AbstractMedia {
    @JsonProperty("num_seasons")
    private int numberOfSeasons;
    private String status;

    @Override
    public boolean isMovie() {
        return false;
    }
}
