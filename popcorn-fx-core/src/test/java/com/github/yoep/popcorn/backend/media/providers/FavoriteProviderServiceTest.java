package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.web.client.RestTemplate;

import java.util.Arrays;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

import static java.util.Collections.emptyList;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class FavoriteProviderServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;

    @Test
    void testSupports_whenTypeIsNotFavorites_shouldReturnFalse() {
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), emptyList());

        var result = service.supports(Category.MOVIES);

        assertFalse(result);
    }

    @Test
    void testSupports_whenTypeIsFavorites_shouldReturnTrue() {
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), emptyList());

        var result = service.supports(Category.FAVORITES);

        assertTrue(result);
    }

    @Test
    void testGetPage_whenPageIsGreaterThan1_shouldReturnEmptyPage() throws ExecutionException, InterruptedException {
        var genre = new Genre("genreKey", "my-genre");
        var sortBy = new SortBy("sortKey", "my-sort");
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), emptyList());

        var result = service.getPage(genre, sortBy, 2);

        assertTrue(result.get().isEmpty(), "Expected an empty page to be returned");
    }

    @Test
    void testRetrieveDetails_whenMediaIsMovie_shouldRetrieveDetailsFromTheMovieProvider() throws ExecutionException, InterruptedException {
        var movie = Movie.builder().build();
        var expectedResult = Movie.builder()
                .title("my-enhanced-movie-details")
                .build();
        var movieProvider = mock(ProviderService.class);
        var showProvider = mock(ProviderService.class);
        var providers = Arrays.<ProviderService<?>>asList(showProvider, movieProvider);
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, providers, emptyList());
        when(movieProvider.supports(Category.MOVIES)).thenReturn(true);
        when(showProvider.supports(Category.MOVIES)).thenReturn(false);
        when(movieProvider.retrieveDetails(movie)).thenReturn(CompletableFuture.completedFuture(expectedResult));

        var completableFuture = service.retrieveDetails(movie);
        var result = completableFuture.get();

        assertEquals(expectedResult, result);
    }
}