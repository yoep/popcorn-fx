package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DetailsComponentServiceTest {
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @Mock
    private ApplicationConfig applicationConfig;
    @InjectMocks
    private DetailsComponentService service;

    @Test
    void testIsWatched_whenInvoked_shouldPassMediaItemToWatchedService() {
        var media = MovieDetails.builder()
                .images(new Images())
                .build();
        var expectedResult = true;
        when(watchedService.isWatched(media)).thenReturn(expectedResult);

        var result = service.isWatched(media);

        verify(watchedService).isWatched(media);
        assertEquals(expectedResult, result);
    }

    @Test
    void testIsLiked_whenInvoked_shouldPassMediaItemToFavoriteService() {
        var media = MovieDetails.builder()
                .images(new Images())
                .build();
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
                .images(new Images())
                .build();
        service.init();

        service.toggleWatchedState(movie);

        verify(watchedService).addToWatchList(movie);
    }

    @Test
    void testToggleLikedState_whenLastItemIsKnownAndStateIsUnliked_shouldAddToFavorites() {
        var show = mock(ShowDetails.class);
        when(favoriteService.isLiked(show)).thenReturn(false);
        service.init();

        service.toggleLikedState(show);

        verify(favoriteService).addToFavorites(show);
    }

    @Test
    void testToggleLikedState_whenLastItemIsKnownAndStateIsLiked_shouldRemoveFromFavorites() {
        var show = mock(ShowDetails.class);
        when(favoriteService.isLiked(show)).thenReturn(true);
        service.init();

        service.toggleLikedState(show);

        verify(favoriteService).removeFromFavorites(show);
    }

    @Test
    void testIsTvMode() {
        when(applicationConfig.isTvMode()).thenReturn(true);

        var result = service.isTvMode();

        assertTrue(result);
    }
}