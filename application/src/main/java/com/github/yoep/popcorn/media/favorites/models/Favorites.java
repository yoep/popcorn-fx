package com.github.yoep.popcorn.media.favorites.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;

@Slf4j
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class Favorites {
    /**
     * The saved movie favorites.
     */
    @Builder.Default
    private List<Movie> movies = new ArrayList<>();
    /**
     * The saved show favorites.
     */
    @Builder.Default
    private List<Show> shows = new ArrayList<>();

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

        if (favorable instanceof Movie) {
            movies.add((Movie) favorable);
        } else if (favorable instanceof Show) {
            shows.add((Show) favorable);
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

        if (favorable instanceof Movie) {
            log.trace("Removing movie favorite {}", favorable);
            movies.removeIf(e -> e.getId().equalsIgnoreCase(favorable.getId()));
        } else if (favorable instanceof Show) {
            log.trace("Removing show favorite {}", favorable);
            shows.removeIf(e -> e.getId().equalsIgnoreCase(favorable.getId()));
        } else {
            log.warn("Unable to remove favorable of type \"{}\"", favorable.getClass().getSimpleName());
        }
    }
}
