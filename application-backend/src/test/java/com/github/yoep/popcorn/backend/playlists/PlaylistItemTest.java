package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;

class PlaylistItemTest {

    @Test
    void testGetUrl() {
        var expectedResult = "https://localhost:98745/myurl";
        var item = new PlaylistItem();
        item.url = null;
        assertEquals(Optional.empty(), item.getUrl());

        item.url = expectedResult;
        assertEquals(Optional.of(expectedResult), item.getUrl());
    }

    @Test
    void testGetMedia() {
        var media = new ShowOverview.ByReference();
        var item = new PlaylistItem();
        assertEquals(Optional.empty(), item.getMedia());

        item.media = new MediaItem.ByReference();
        item.media.showOverview = media;
        assertEquals(Optional.of(media), item.getMedia());
    }

    @Test
    void testClose() {
        var item = new PlaylistItem();
        item.close();
    }

    @Test
    void testGetFromMediaTrailer() {
        var movie = new MovieDetails.ByReference();
        movie.title = "FooBar";
        movie.trailer = "https://youtube.com";

        var result = PlaylistItem.fromMediaTrailer(movie);

        assertEquals(movie.title, result.title);
        assertEquals(movie.trailer, result.url);
        assertEquals(movie, result.media.movieDetails);
    }
}