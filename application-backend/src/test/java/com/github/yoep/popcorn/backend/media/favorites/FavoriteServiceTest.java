package com.github.yoep.popcorn.backend.media.favorites;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.backend.media.favorites.models.Favorites;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.utils.FileService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.task.SyncTaskExecutor;
import org.springframework.core.task.TaskExecutor;

import java.io.File;
import java.io.IOException;
import java.time.LocalDateTime;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FavoriteServiceTest {
    @Mock
    private ObjectMapper objectMapper;
    @Spy
    private TaskExecutor taskExecutor = new SyncTaskExecutor();
    @Mock
    private FileService fileService;
    @Mock
    private ProviderService<Movie> movieProviderService;
    @Mock
    private ProviderService<Show> showProviderService;
    @InjectMocks
    private FavoriteService favoriteService;

    @Test
    void testIsCacheUpdateRequired_whenCacheLastUpdateIsBeforeCacheUpdateThreshold_shouldNotUpdateTheCache() throws IOException {
        var favorites = mock(Favorites.class);
        var file = mock(File.class);
        var lastUpdate = LocalDateTime.now().minusHours(FavoriteService.UPDATE_CACHE_AFTER_HOURS - 1);
        when(favorites.getLastCacheUpdate()).thenReturn(lastUpdate);
        when(file.exists()).thenReturn(true);
        when(fileService.getFile(isA(String.class))).thenReturn(file);
        when(objectMapper.readValue(isA(File.class), eq(Favorites.class))).thenReturn(favorites);

        favoriteService.init();

        verify(taskExecutor, times(0)).execute(isA(Runnable.class));
    }

    @Test
    void testIsCacheUpdateRequired_whenCacheLastUpdateIsAfterCacheUpdateThreshold_shouldUpdateTheCache() throws IOException {
        var favorites = mock(Favorites.class);
        var file = mock(File.class);
        var lastUpdate = LocalDateTime.now().minusHours(FavoriteService.UPDATE_CACHE_AFTER_HOURS + 1);
        when(favorites.getLastCacheUpdate()).thenReturn(lastUpdate);
        when(file.exists()).thenReturn(true);
        when(fileService.getFile(isA(String.class))).thenReturn(file);
        when(objectMapper.readValue(isA(File.class), eq(Favorites.class))).thenReturn(favorites);

        favoriteService.init();

        verify(taskExecutor).execute(isA(Runnable.class));
    }

    @Test
    void testIsCacheUpdateRequired_whenCacheLastUpdateIsNull_shouldUpdateTheCache() throws IOException {
        var favorites = mock(Favorites.class);
        var file = mock(File.class);
        when(favorites.getLastCacheUpdate()).thenReturn(null);
        when(file.exists()).thenReturn(true);
        when(fileService.getFile(isA(String.class))).thenReturn(file);
        when(objectMapper.readValue(isA(File.class), eq(Favorites.class))).thenReturn(favorites);

        favoriteService.init();

        verify(taskExecutor).execute(isA(Runnable.class));
    }
}
