package com.github.yoep.popcorn.media.providers.models;

import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

import java.util.List;

@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
@Data
public class Show extends AbstractMedia {
    @JsonProperty("num_seasons")
    private int numberOfSeasons;
    private String status;
    private List<Episode> episodes;

    @Override
    public boolean isMovie() {
        return false;
    }
}
