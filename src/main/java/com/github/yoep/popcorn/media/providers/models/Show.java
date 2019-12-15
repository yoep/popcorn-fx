package com.github.yoep.popcorn.media.providers.models;

import com.fasterxml.jackson.annotation.JsonProperty;

public class Show extends AbstractMedia {
    @JsonProperty("num_seasons")
    private int numberOfSeasons;

    @Override
    public boolean isMovie() {
        return false;
    }
}
