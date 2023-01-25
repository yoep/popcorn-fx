package com.github.yoep.popcorn.backend.media.providers.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Data
@ToString
@EqualsAndHashCode(callSuper = false, exclude = {"rating"})
@NoArgsConstructor
@AllArgsConstructor
@JsonIgnoreProperties({"autoAllocate", "stringEncoding", "typeMapper", "fields", "pointer"})
@Structure.FieldOrder({"title", "imdbId", "year", "rating", "images"})
public class MovieOverview extends Structure implements Media, Closeable {
    public static class ByReference extends MovieOverview implements Structure.ByReference {
    }

    public String title;
    public String imdbId;
    public String year;
    public Rating.ByReference rating;
    public Images images;

    //region Properties

    @Override
    public MediaType getType() {
        return MediaType.MOVIE;
    }

    //endregion

    @Override
    public String getId() {
        return imdbId;
    }

    @Override
    public String getSynopsis() {
        return "";
    }

    @Override
    public Integer getRuntime() {
        return 0;
    }

    @Override
    @JsonIgnore
    public List<String> getGenres() {
        return Collections.emptyList();
    }

    public Optional<Rating> getRating() {
        return Optional.ofNullable(rating);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
