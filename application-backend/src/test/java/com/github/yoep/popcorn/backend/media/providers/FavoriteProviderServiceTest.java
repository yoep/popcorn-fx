package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.ExecutionException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class FavoriteProviderServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private FavoriteProviderService service;

    @Test
    void testSupports() {
        assertTrue(service.supports(Category.FAVORITES), "expected favorites to be supported");
        assertFalse(service.supports(Category.MOVIES), "expected movies to not be supported");
    }

    @Test
    void testRetrieveDetails() throws ExecutionException, InterruptedException {
        var imdbId = "tt121212";
        var overview = new ShowOverview();
        var details = new ShowDetails();
        var mediaItem = mock(MediaItem.class);
        overview.imdbId = imdbId;
        doAnswer(invocation -> {
            fxLib.dispose_media_item(mediaItem);
            return null;
        }).when(mediaItem).close();
        when(mediaItem.getMedia()).thenReturn(details);
        when(fxLib.retrieve_favorite_details(instance, imdbId)).thenReturn(mediaItem);

        var result = service.retrieveDetails(overview);

        assertEquals(details, result.get());
        verify(fxLib).dispose_media_item(mediaItem);
    }
}