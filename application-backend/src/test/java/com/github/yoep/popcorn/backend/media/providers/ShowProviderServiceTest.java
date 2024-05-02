package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaError;
import com.github.yoep.popcorn.backend.media.MediaSet;
import com.github.yoep.popcorn.backend.media.MediaSetResult;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Collections;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.fail;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class ShowProviderServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Spy
    private ExecutorService executorService = Executors.newSingleThreadExecutor();
    @InjectMocks
    private ShowProviderService provider;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testGetPage() throws ExecutionException, InterruptedException {
        var mediaResult = new MediaSetResult.ByValue();
        var mediaSet = mock(MediaSet.ByValue.class);
        var genre = new Genre("lorem", "");
        var sortBy = new SortBy("ipsum", "");
        var expectedResult = Collections.singletonList(mock(ShowOverview.class));
        mediaResult.tag = MediaSetResult.Tag.Ok;
        mediaResult.union = new MediaSetResult.MediaSetResultUnion.ByValue();
        mediaResult.union.ok = new MediaSetResult.OkBody();
        mediaResult.union.ok.mediaSet = mediaSet;
        when(mediaSet.getShows()).thenReturn(expectedResult);
        when(fxLib.retrieve_available_shows(instance, genre, sortBy, "", 1)).thenReturn(mediaResult);

        var result = provider.getPage(genre, sortBy, 1).get();

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetPageNoAvailableProviders() {
        var mediaResult = new MediaSetResult.ByValue();
        var genre = new Genre("lorem", "");
        var sortBy = new SortBy("ipsum", "");
        mediaResult.tag = MediaSetResult.Tag.Err;
        mediaResult.union = new MediaSetResult.MediaSetResultUnion.ByValue();
        mediaResult.union.err = new MediaSetResult.ErrBody();
        mediaResult.union.err.mediaError = MediaError.NoAvailableProviders;
        when(fxLib.retrieve_available_shows(instance, genre, sortBy, "", 1)).thenReturn(mediaResult);

        try {
            provider.getPage(genre, sortBy, 1).get();
            fail("expected an exception");
        } catch (Exception e) {
            assertEquals(MediaRetrievalException.class, e.getCause().getClass());
        }
    }

    @Test
    void testGetPageNoItemsFound() throws ExecutionException, InterruptedException {
        var mediaResult = new MediaSetResult.ByValue();
        var genre = new Genre("lorem", "");
        var sortBy = new SortBy("ipsum", "");
        mediaResult.tag = MediaSetResult.Tag.Err;
        mediaResult.union = new MediaSetResult.MediaSetResultUnion.ByValue();
        mediaResult.union.err = new MediaSetResult.ErrBody();
        mediaResult.union.err.mediaError = MediaError.NoItemsFound;
        when(fxLib.retrieve_available_shows(instance, genre, sortBy, "", 1)).thenReturn(mediaResult);

        var result = provider.getPage(genre, sortBy, 1).get();

        assertEquals(Collections.emptyList(), result);
    }

    @Test
    void testGetPageFailed() {
        var mediaResult = new MediaSetResult.ByValue();
        var genre = new Genre("ipsum", "");
        var sortBy = new SortBy("dolor", "");
        mediaResult.tag = MediaSetResult.Tag.Err;
        mediaResult.union = new MediaSetResult.MediaSetResultUnion.ByValue();
        mediaResult.union.err = new MediaSetResult.ErrBody();
        mediaResult.union.err.mediaError = MediaError.Failed;
        when(fxLib.retrieve_available_shows(instance, genre, sortBy, "", 1)).thenReturn(mediaResult);

        try {
            provider.getPage(genre, sortBy, 1).get();
            fail("expected an exception");
        } catch (Exception e) {
            assertEquals(MediaException.class, e.getCause().getClass());
        }
    }
}