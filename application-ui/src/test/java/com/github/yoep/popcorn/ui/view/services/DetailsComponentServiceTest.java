package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertFalse;

@ExtendWith(MockitoExtension.class)
class DetailsComponentServiceTest {
    @Mock
    private FavoriteService favoriteService;
    @Mock
    private WatchedService watchedService;
    @InjectMocks
    private DetailsComponentService service;

    @Test
    void testListeners_whenWatchedIsChanged_shouldInvokedOnWatchedChanged() {
        assertFalse(true, "implement!");
    }
}