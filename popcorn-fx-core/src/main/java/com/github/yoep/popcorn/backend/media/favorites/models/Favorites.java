package com.github.yoep.popcorn.backend.media.favorites.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.Serializable;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.List;

@Slf4j
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class Favorites implements Serializable {
    /**
     * The saved movie favorites.
     */
    @Builder.Default
    private List<MovieOverview> movies = new ArrayList<>();
    /**
     * The saved show favorites.
     */
    @Builder.Default
    private List<ShowOverview> shows = new ArrayList<>();
    /**
     * The last time the favorites cache has been updated.
     */
    private LocalDateTime lastCacheUpdate;

    /**
     * Get all the favorites.
     *
     * @return Returns all the favorites.
     */
    @JsonIgnore
    public List<Favorable> getAll() {
        List<Favorable> mediaList = new ArrayList<>(movies);
        mediaList.addAll(shows);
        return mediaList;
    }

    /**
     * Add the given {@link Favorable} to the favorites.
     *
     * @param favorable The favorable to add.
     */
    public void add(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");

        if (favorable instanceof MovieDetails) {
            movies.add((MovieDetails) favorable);
        } else if (favorable instanceof ShowOverview) {
            shows.add((ShowOverview) favorable);
        } else {
            log.warn("Unable to add favorable of type \"{}\"", favorable.getClass().getSimpleName());
        }
    }

    /**
     * Remove the given {@link Favorable} from favorites.
     *
     * @param favorable The favorable to remove.
     */
    public void remove(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");

        if (favorable instanceof MovieDetails) {
            log.trace("Removing movie favorite {}", favorable);
            movies.removeIf(e -> e.getId().equalsIgnoreCase(favorable.getId()));
        } else if (favorable instanceof ShowDetails) {
            log.trace("Removing show favorite {}", favorable);
            shows.removeIf(e -> e.getId().equalsIgnoreCase(favorable.getId()));
        } else {
            log.warn("Unable to remove favorable of type \"{}\"", favorable.getClass().getSimpleName());
        }
    }
}
