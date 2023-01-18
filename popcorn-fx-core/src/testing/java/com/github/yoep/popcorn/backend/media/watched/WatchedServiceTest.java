package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.watched.models.Watched;
import com.github.yoep.popcorn.backend.storage.StorageService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.Optional;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class WatchedServiceTest {
    @Mock
    private StorageService storageService;
    @InjectMocks
    private WatchedService watchedService;

    @Test
    void testIsWatched_whenItemHasBeenWatched_shouldReturnTrue() {
        var id = "myMovieId";
        var movie = MovieDetails.builder()
                .imdbId(id)
                .build();
        var watchedItems = Watched.builder()
                .movies(Collections.singletonList(id))
                .build();
        when(storageService.read(WatchedService.STORAGE_NAME, Watched.class)).thenReturn(Optional.of(watchedItems));

        var result = watchedService.isWatched(movie);

        assertTrue(result, "Expected the watchable to have been watched");
    }

    @Test
    void testGetWatchedMovies_whenInvoked_shouldReturnTheWatchedItems() {
        var expectedResult = asList("lorem", "ipsum", "dolor");
        var watchedItems = Watched.builder()
                .movies(expectedResult)
                .build();
        when(storageService.read(WatchedService.STORAGE_NAME, Watched.class)).thenReturn(Optional.of(watchedItems));

        var result = watchedService.getWatchedMovies();

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetWatchedShows_whenInvoked_shouldReturnTheWatchedItems() {
        var expectedResult = asList("lorem", "ipsum", "estla");
        var watchedItems = Watched.builder()
                .shows(expectedResult)
                .build();
        when(storageService.read(WatchedService.STORAGE_NAME, Watched.class)).thenReturn(Optional.of(watchedItems));

        var result = watchedService.getWatchedShows();

        assertEquals(expectedResult, result);
    }

    @Test
    void testOnDestroy_whenInvoked_shouldSaveCache() {
        var id = "movieId";
        var movie = MovieDetails.builder()
                .imdbId(id)
                .build();
        var expectedStorageItem = Watched.builder()
                .movies(Collections.singletonList(id))
                .shows(Collections.emptyList())
                .build();
        watchedService.addToWatchList(movie);

        watchedService.destroy();

        verify(storageService).store(WatchedService.STORAGE_NAME, expectedStorageItem);
    }
}