package com.github.yoep.popcorn.backend.media.filters.model;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaType;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import lombok.AllArgsConstructor;
import lombok.Data;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@AllArgsConstructor
public class Season implements Media {
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
    @JsonIgnore
    public MediaType getType() {
        return MediaType.SHOW;
    }

    //endregion

    @Override
    public String toString() {
        return text;
    }
}