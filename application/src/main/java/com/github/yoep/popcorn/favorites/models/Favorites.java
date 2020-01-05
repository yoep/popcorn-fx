package com.github.yoep.popcorn.favorites.models;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.providers.models.Movie;
import com.github.yoep.popcorn.providers.models.Show;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.ArrayList;
import java.util.List;

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
    public List<Media> getAll() {
        List<Media> mediaList = new ArrayList<>(movies);
        mediaList.addAll(shows);
        return mediaList;
    }

    /**
     * Add the given media to the favorites.
     *
     * @param media The media to add.
     */
    public void add(Media media) {
        if (media instanceof Movie) {
            movies.add((Movie) media);
        } else {
            shows.add((Show) media);
        }
    }

    /**
     * Remove the given media from favorites.
     *
     * @param media The media to remove.
     */
    public void remove(Media media) {
        if (media instanceof Movie) {
            movies.remove(media);
        } else {
            shows.remove(media);
        }
    }
}
