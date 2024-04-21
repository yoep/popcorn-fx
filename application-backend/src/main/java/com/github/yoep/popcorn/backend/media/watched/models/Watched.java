package com.github.yoep.popcorn.backend.media.watched.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class Watched {
    /**
     * The ID's of the watched movies.
     */
    @Builder.Default
    private List<String> movies = new ArrayList<>();
    /**
     * The ID's of the watched shows.
     */
    @Builder.Default
    private List<String> shows = new ArrayList<>();

    /**
     * Check if the given key is a watched item
     *
     * @return Returns true if the key has been watched, else false.
     */
    public boolean contains(String key) {
        return movies.contains(key) || shows.contains(key);
    }

    /**
     * Add the given movie key to the watched items.
     *
     * @param key The key to add.
     */
    public void addMovie(String key) {
        Objects.requireNonNull(key, "key cannot be null");
        movies.add(key);
    }

    /**
     * Add the given show/episode key to the watched items.
     *
     * @param key The key to add.
     */
    public void addShow(String key) {
        Objects.requireNonNull(key, "key cannot be null");
        shows.add(key);
    }

    /**
     * Remove the given key from the watched items.
     *
     * @param key The key to remove.
     */
    public void remove(String key) {
        movies.remove(key);
        shows.remove(key);
    }
}
