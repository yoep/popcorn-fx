package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.lib.StringArray;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class WatchedServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private WatchedService service;

    @Test
    void testIsWatched() {
        var imdbId = "tt0000132";
        var media = new ShowOverview.ByReference();
        media.imdbId = imdbId;
        var mediaItem = MediaItem.from(media);
        when(fxLib.is_media_watched(instance, mediaItem)).thenReturn((byte) 1);

        var result = service.isWatched(media);

        assertTrue(result);
    }

    @Test
    void testGetWatchedMovies() {
        var movies = mock(StringArray.class);
        var expectedResult = asList("tt1111", "tt2222");
        when(movies.values()).thenReturn(expectedResult);
        when(fxLib.retrieve_watched_movies(instance)).thenReturn(movies);

        var result = service.getWatchedMovies();

        assertArrayEquals(expectedResult.toArray(), result.toArray());
    }

    @Test
    void testGetWatchedShows() {
        var shows = mock(StringArray.class);
        var expectedResult = asList("tt7410", "tt8520");
        when(shows.values()).thenReturn(expectedResult);
        when(fxLib.retrieve_watched_shows(instance)).thenReturn(shows);

        var result = service.getWatchedShows();

        assertArrayEquals(expectedResult.toArray(), result.toArray());
    }
}