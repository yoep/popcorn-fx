package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.media.favorites.models.Favorites;
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
import java.util.Optional;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FavoriteServiceTest {
    @Spy
    private TaskExecutor taskExecutor = new SyncTaskExecutor();
    @Mock
    private StorageService storageService;
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
}
