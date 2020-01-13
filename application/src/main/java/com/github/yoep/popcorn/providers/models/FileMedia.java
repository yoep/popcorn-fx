package com.github.yoep.popcorn.providers.models;

import com.github.yoep.popcorn.watched.models.AbstractWatchable;
import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;

import java.util.Collections;
import java.util.List;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
public class FileMedia extends AbstractWatchable implements Media {
    private String title;
    private Integer runtime;

    @Override
    public String getId() {
        return null;
    }

    @Override
    public MediaType getType() {
        return MediaType.UNKNOWN;
    }

    @Override
    public String getSynopsis() {
        return null;
    }

    @Override
    public String getYear() {
        return null;
    }

    @Override
    public List<String> getGenres() {
        return Collections.emptyList();
    }

    @Override
    public Rating getRating() {
        return new Rating();
    }

    @Override
    public Images getImages() {
        return new Images();
    }
}
