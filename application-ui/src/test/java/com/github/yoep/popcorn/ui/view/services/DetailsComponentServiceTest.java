package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DetailsComponentServiceTest {
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @InjectMocks
    private DetailsComponentService service;

    @Test
    void testIsWatched_whenInvoked_shouldPassMediaItemToWatchedService() {
        var media = MovieDetails.builder().build();
        var expectedResult = true;
        when(watchedService.isWatched(media)).thenReturn(expectedResult);

        var result = service.isWatched(media);

        verify(watchedService).isWatched(media);
        assertEquals(expectedResult, result);
    }

    @Test
    void testIsLiked_whenInvoked_shouldPassMediaItemToFavoriteService() {
        var media = MovieDetails.builder().build();
        var expectedResult = false;
        when(favoriteService.isLiked(media)).thenReturn(expectedResult);

        var result = service.isLiked(media);

        verify(favoriteService).isLiked(media);
        assertEquals(expectedResult, result);
    }

    @Test
    void testUpdateWatchedStated_whenMediaItemIsGivenAndIsWatched_shouldAddToWatchlist() {
        var media = mock(Media.class);

        service.updateWatchedStated(media, true);

        verify(watchedService).addToWatchList(media);
    }

    @Test
    void testUpdateWatchedStated_whenMediaItemIsGivenAndMarkAsUnseen_shouldRemoveFromWatchlist() {
        var media = mock(Media.class);

        service.updateWatchedStated(media, false);

        verify(watchedService).removeFromWatchList(media);
    }

    @Test
    void testToggleWatchedState_whenLastItemIsKnownAndStateIsNotSeen_shouldAddToWatchlist() {
        var movie = MovieDetails.builder()
                .build();
        var event = ShowMovieDetailsEvent.builder()
                .source(this)
                .media(movie)
                .build();
        service.onShowDetails(event);

        service.toggleWatchedState();

        verify(watchedService).addToWatchList(movie);
    }

    @Test
    void testToggleLikedState_whenLastItemIsKnownAndStateIsUnliked_shouldAddToFavorites() {
        var show = mock(ShowDetails.class);
        var event = ShowSerieDetailsEvent.builder()
                .source(this)
                .media(show)
                .build();
        when(favoriteService.isLiked(show)).thenReturn(false);
        service.onShowDetails(event);

        service.toggleLikedState();

        verify(favoriteService).addToFavorites(show);
    }

    @Test
    void testToggleLikedState_whenLastItemIsKnownAndStateIsLiked_shouldRemoveFromFavorites() {
        var show = mock(ShowDetails.class);
        var event = ShowSerieDetailsEvent.builder()
                .source(this)
                .media(show)
                .build();
        when(favoriteService.isLiked(show)).thenReturn(true);
        service.onShowDetails(event);

        service.toggleLikedState();

        verify(favoriteService).removeFromFavorites(show);
    }
}