package com.github.yoep.popcorn.backend.media.filters.model;

import com.github.yoep.popcorn.backend.media.providers.Images;
import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaType;
import com.github.yoep.popcorn.backend.media.providers.Rating;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.EqualsAndHashCode;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@AllArgsConstructor
@EqualsAndHashCode
public class Season implements Media, Comparable {
    private final int season;
    private final String text;

    //region Getters

    @Override
    public String getId() {
        return null;
    }

    @Override
    public String getTitle() {
        return text;
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
    public Integer getRuntime() {
        return null;
    }

    @Override
    public List<String> getGenres() {
        return Collections.emptyList();
    }

    @Override
    public Optional<Rating> getRating() {
        return Optional.empty();
    }

    @Override
    public Images getImages() {
        return null;
    }

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }

    //endregion

    @Override
    public String toString() {
        return text;
    }

    @Override
    public int compareTo(Object other) {
        return Optional.ofNullable(other)
                .filter(e -> e instanceof Season)
                .map(e -> (Season) e)
                .map(e -> Integer.compare(season, e.season))
                .orElse(0);
    }
}
