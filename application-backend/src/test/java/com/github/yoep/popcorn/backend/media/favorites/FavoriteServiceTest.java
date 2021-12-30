package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.media.favorites.models.Favorites;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.storage.StorageService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.task.SyncTaskExecutor;
import org.springframework.core.task.TaskExecutor;

import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FavoriteServiceTest {
    @Spy
    private TaskExecutor taskExecutor = new SyncTaskExecutor();
    @Mock
    private StorageService storageService;
    @Mock
    private ProviderService<Movie> movieProviderService;
    @Mock
    private ProviderService<Show> showProviderService;
    @InjectMocks
    private FavoriteService favoriteService;

    @Test
    void testIsCacheUpdateRequired_whenCacheLastUpdateIsBeforeCacheUpdateThreshold_shouldNotUpdateTheCache() {
        var favorites = mock(Favorites.class);
        var lastUpdate = LocalDateTime.now().minusHours(FavoriteService.UPDATE_CACHE_AFTER_HOURS - 1);
        when(favorites.getLastCacheUpdate()).thenReturn(lastUpdate);
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        favoriteService.init();

        verify(taskExecutor, times(0)).execute(isA(Runnable.class));
    }

    @Test
    void testIsCacheUpdateRequired_whenCacheLastUpdateIsAfterCacheUpdateThreshold_shouldUpdateTheCache() {
        var favorites = mock(Favorites.class);
        var lastUpdate = LocalDateTime.now().minusHours(FavoriteService.UPDATE_CACHE_AFTER_HOURS + 1);
        when(favorites.getLastCacheUpdate()).thenReturn(lastUpdate);
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        favoriteService.init();

        verify(taskExecutor).execute(isA(Runnable.class));
    }

    @Test
    void testIsCacheUpdateRequired_whenCacheLastUpdateIsNull_shouldUpdateTheCache() {
        var favorites = mock(Favorites.class);
        when(favorites.getLastCacheUpdate()).thenReturn(null);
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        favoriteService.init();

        verify(taskExecutor).execute(isA(Runnable.class));
    }

    @Test
    void testAddToFavorites_whenInvoked_shouldAddTheItemToTheList() {
        var movie = Movie.builder()
                .id("movieId")
                .title("movieTitle")
                .year("movieYear")
                .build();
        var favorites = Favorites.builder()
                .build();
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        favoriteService.addToFavorites(movie);

        assertEquals(1, favorites.getMovies().size());
        assertEquals(movie, favorites.getMovies().get(0));
    }

    @Test
    void testRemoveFromFavorites_whenInvoked_shouldAddTheItemToTheList() {
        var show = Show.builder()
                .id("showId")
                .title("showTitle")
                .year("showYear")
                .imdbId("showImdbId")
                .numberOfSeasons(5)
                .build();
        var favorites = Favorites.builder()
                .shows(new ArrayList<>(Collections.singletonList(show)))
                .build();
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        favoriteService.removeFromFavorites(show);

        assertEquals(0, favorites.getShows().size());
    }

    @Test
    void testGetAll_whenFavoritesExist_shouldReturnTheListFromTheStorage() {
        var show = Show.builder()
                .id("showId")
                .title("showTitle")
                .year("showYear")
                .imdbId("showImdbId")
                .numberOfSeasons(10)
                .build();
        var favorites = Favorites.builder()
                .shows(new ArrayList<>(Collections.singletonList(show)))
                .build();
        var expectedResult = Collections.singletonList(show);
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        var result = favoriteService.getAll();

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetAll_whenFavoritesDoNotExist_shouldReturnNewFavorites() {
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.empty());

        var result = favoriteService.getAll();

        assertEquals(0, result.size());
    }

    @Test
    void testIsLiked_whenItemIsStored_shouldReturnTrue() {
        var likedId = "showId";
        var storedShow = Show.builder()
                .id(likedId)
                .title("showTitle")
                .year("showYear")
                .imdbId("showImdbId")
                .numberOfSeasons(10)
                .build();
        var show = Show.builder()
                .id(likedId)
                .build();
        var favorites = Favorites.builder()
                .shows(new ArrayList<>(Collections.singletonList(storedShow)))
                .build();
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        var result = favoriteService.isLiked(show);

        assertTrue(result);
    }

    @Test
    void testIsLiked_whenItemIsNotStored_shouldReturnFalse() {
        var likedId = "showId";
        var show = Show.builder()
                .id(likedId)
                .build();
        var favorites = Favorites.builder()
                .shows(new ArrayList<>())
                .build();
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.of(favorites));

        var result = favoriteService.isLiked(show);

        assertFalse(result);
    }

    @Test
    void testDestroy_whenCacheContainsItems_shouldStoreTheItemsToTheStorage() {
        var movieId = "myMovieId";
        var movie = Movie.builder()
                .id(movieId)
                .build();
        var expectedFavoritesToStore = Favorites.builder()
                .movies(Collections.singletonList(movie))
                .build();
        when(storageService.read(FavoriteService.STORAGE_NAME, Favorites.class)).thenReturn(Optional.empty());

        favoriteService.addToFavorites(movie);
        favoriteService.destroy();

        verify(storageService).store(FavoriteService.STORAGE_NAME, expectedFavoritesToStore);
    }
}
