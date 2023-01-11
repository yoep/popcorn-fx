package com.github.yoep.popcorn.backend.media.favorites.models;

import com.github.yoep.popcorn.backend.media.providers.models.MediaType;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import javafx.beans.property.BooleanProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FavoritesTest {
    private Favorites favorites;

    @BeforeEach
    void setUp() {
        favorites = new Favorites();
    }

    @Test
    void testAdd_whenFavorableIsEmpty_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> favorites.add(null), "favorable cannot be null");
    }

    @Test
    void testAdd_whenFavorableIsMovie_shouldAddItemToTheMoviesList() {
        var media = Movie.builder().build();

        favorites.add(media);
        var result = favorites.getMovies();

        assertTrue(result.contains(media), "Favorable should have been added to the movies list");
    }

    @Test
    void testAdd_whenFavorableIsShow_shouldAddItemToTheShowsList() {
        var media = new ShowOverview();

        favorites.add(media);
        var result = favorites.getShows();

        assertTrue(result.contains(media), "Favorable should have been added to the shows list");
    }

    @Test
    void testAdd_whenFavorableIsUnknownType_shouldIgnoreTheItem() {
        var media = new RandomFavorable();

        favorites.add(media);
        var result = favorites.getAll();

        assertFalse(result.contains(media), "Favorable should have been ignored");
    }

    static class RandomFavorable implements Favorable {

        @Override
        public boolean isLiked() {
            return false;
        }

        @Override
        public BooleanProperty likedProperty() {
            return null;
        }

        @Override
        public void setLiked(boolean liked) {

        }

        @Override
        public String getId() {
            return null;
        }

        @Override
        public MediaType getType() {
            return null;
        }
    }
}
