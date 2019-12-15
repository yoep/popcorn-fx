package com.github.yoep.popcorn.media.providers.models;

import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.ToString;

@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
@Data
public class Movie extends AbstractMedia {
    private String trailer;

    @Override
    public boolean isMovie() {
        return true;
    }
}
