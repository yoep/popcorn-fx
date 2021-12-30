package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteSortStrategy;
import com.github.yoep.popcorn.backend.media.favorites.models.Favorable;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.web.client.RestTemplate;

import java.util.Arrays;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

import static java.util.Collections.emptyList;
import static java.util.Collections.singletonList;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.any;
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
    void testGetPage_whenSortIsGiven_shouldSortTheFavoritesThroughTheSortStrategy() throws ExecutionException, InterruptedException {
        var genre = new Genre(Genre.ALL_KEYWORD, "my-genre");
        var sortBy = new SortBy("sortKey", "my-sort");
        var favorites = Arrays.<Favorable>asList(Movie.builder()
                .id("movie1")
                .build(), Show.builder()
                .id("myShow")
                .build(), Movie.builder()
                .id("movie2")
                .build());
        var expectedResult = Arrays.<Media>asList(Movie.builder()
                .id("movie1")
                .build(), Movie.builder()
                .id("movie2")
                .build(), Show.builder()
                .id("showId")
                .build());
        var sortStrategy = mock(FavoriteSortStrategy.class);
        var sortStrategies = singletonList(sortStrategy);
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), sortStrategies);
        when(favoriteService.getAll()).thenReturn(favorites);
        when(sortStrategy.support(sortBy)).thenReturn(true);
        when(sortStrategy.sort(any())).thenReturn(expectedResult.stream());

        var completableFuture = service.getPage(genre, sortBy, 1);

        var result = completableFuture.get();
        assertTrue(result.hasContent(), "Expected media content to be returned");
        assertEquals(expectedResult, result.getContent());
    }

    @Test
    void testGetPage_whenGenreIsMovies_shouldFilterOutAllShows() throws ExecutionException, InterruptedException {
        var genre = new Genre(Genre.MOVIES_KEYWORD, "movies-only");
        var sortBy = new SortBy("sortKey", "my-sort");
        var favorites = Arrays.<Favorable>asList(Movie.builder()
                .id("movie1")
                .build(), Show.builder()
                .id("myShow")
                .build(), Movie.builder()
                .id("movie2")
                .build());
        var expectedResult = Arrays.<Media>asList(Movie.builder()
                .id("movie1")
                .build(), Movie.builder()
                .id("movie2")
                .build());
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), emptyList());
        when(favoriteService.getAll()).thenReturn(favorites);

        var completableFuture = service.getPage(genre, sortBy, 1);

        var result = completableFuture.get();
        assertTrue(result.hasContent(), "Expected media content to be returned");
        assertEquals(expectedResult, result.getContent());
    }

    @Test
    void testGetPage_whenGenreIsShows_shouldFilterOutAllMovies() throws ExecutionException, InterruptedException {
        var genre = new Genre("shows", "shows-only");
        var sortBy = new SortBy("sortKey", "my-sort");
        var favorites = Arrays.<Favorable>asList(Movie.builder()
                .id("movie1")
                .build(), Show.builder()
                .id("myShow")
                .build(), Movie.builder()
                .id("movie2")
                .build());
        var expectedResult = List.<Media>of(Show.builder()
                .id("myShow")
                .build());
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), emptyList());
        when(favoriteService.getAll()).thenReturn(favorites);

        var completableFuture = service.getPage(genre, sortBy, 1);

        var result = completableFuture.get();
        assertTrue(result.hasContent(), "Expected media content to be returned");
        assertEquals(expectedResult, result.getContent());
    }

    @Test
    void testGetPage_whenKeywordsAreGiven_shouldFilterMediaItemsMatchingOnlyTheGivenKeywords() throws ExecutionException, InterruptedException {
        var genre = new Genre(Genre.ALL_KEYWORD, "all");
        var sortBy = new SortBy("sortKey", "my-sort");
        var movie1 = Movie.builder()
                .id("movie1")
                .title("lorem")
                .build();
        var movie2 = Movie.builder()
                .id("movie2")
                .title("ipsum")
                .build();
        var show = Show.builder()
                .id("myShow")
                .title("lorem ipsum")
                .build();
        var favorites = Arrays.<Favorable>asList(movie1, show, movie2);
        var expectedResult = Arrays.<Media>asList(movie1, show);
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, emptyList(), emptyList());
        when(favoriteService.getAll()).thenReturn(favorites);

        var completableFuture = service.getPage(genre, sortBy, 1, "lorem");

        var result = completableFuture.get();
        assertTrue(result.hasContent(), "Expected media content to be returned");
        assertEquals(expectedResult, result.getContent());
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

    @Test
    void testRetrieveDetails_whenMediaIsShow_shouldRetrieveDetailsFromTheShowProvider() throws ExecutionException, InterruptedException {
        var show = Show.builder().build();
        var expectedResult = Show.builder()
                .title("my-enhanced-movie-details")
                .build();
        var movieProvider = mock(ProviderService.class);
        var showProvider = mock(ProviderService.class);
        var providers = Arrays.<ProviderService<?>>asList(movieProvider, showProvider);
        var service = new FavoriteProviderService(restTemplate, favoriteService, watchedService, providers, emptyList());
        when(movieProvider.supports(Category.SERIES)).thenReturn(false);
        when(showProvider.supports(Category.SERIES)).thenReturn(true);
        when(showProvider.retrieveDetails(show)).thenReturn(CompletableFuture.completedFuture(expectedResult));

        var completableFuture = service.retrieveDetails(show);
        var result = completableFuture.get();

        assertEquals(expectedResult, result);
    }
}