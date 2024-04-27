package com.github.yoep.popcorn.backend.media.providers.models;

import com.sun.jna.Structure;
import lombok.AllArgsConstructor;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.ToString;

import java.io.Closeable;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Getter
@ToString
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"imdbId", "tvdbId", "title", "year", "numberOfSeasons", "images", "rating"})
public class ShowOverview extends Structure implements Media, Closeable {
    public static class ByReference extends ShowOverview implements Structure.ByReference {
    }

    public String imdbId;
    public String tvdbId;
    public String title;
    public String year;
    public int numberOfSeasons;
    public Images images;
    public Rating.ByReference rating;

    //region Properties

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }

    //endregion

    public String getId() {
        return imdbId;
    }

    public Optional<Rating> getRating() {
        return Optional.ofNullable(rating);
    }

    @Override
    public String getSynopsis() {
        return "";
    }

    @Override
    public Integer getRuntime() {
        return 0;
    }

    public int getSeasons() {
        return numberOfSeasons;
    }

    @Override
    public List<String> getGenres() {
        return new ArrayList<>();
    }

    @Override
    public void close() {
        setAutoSynch(false);
        Optional.ofNullable(rating)
                .ifPresent(Rating::close);
        Optional.ofNullable(images)
                .ifPresent(Images::close);
    }
}
