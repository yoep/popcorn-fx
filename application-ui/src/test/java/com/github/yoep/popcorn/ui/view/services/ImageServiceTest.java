package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.lib.ByteArray;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.Images;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.ShowDetails;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ImageServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Mock
    private ExecutorService executorService;
    @InjectMocks
    private ImageService imageService;

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            var runnable = invocation.getArgument(0, Runnable.class);
            runnable.run();
            return null;
        }).when(executorService).execute(isA(Runnable.class));
    }

    @Test
    void testGetPosterPlaceholder() {
        var byteArray = mock(ByteArray.class);
        when(byteArray.getBytes()).thenReturn(new byte[0]);
        when(fxLib.poster_placeholder(instance)).thenReturn(byteArray);

        var result = imageService.getPosterPlaceholder();

        assertNotNull(result);
    }

    @Test
    void testGetArtworkPlaceholder() {
        var byteArray = mock(ByteArray.class);
        when(byteArray.getBytes()).thenReturn(new byte[0]);
        when(fxLib.artwork_placeholder(instance)).thenReturn(byteArray);

        var result = imageService.getArtPlaceholder();

        assertNotNull(result);
    }

    @Test
    void testLoadFanartException() throws ExecutionException, InterruptedException {
        var url = "http://my-fanart-url.com";
        var images = Images.builder()
                .fanart(url)
                .build();
        var media = createMovie(images);
        when(fxLib.load_fanart(eq(instance), isA(MediaItem.class))).thenThrow(new RuntimeException("my exception"));

        var result = imageService.loadFanart(media);

        assertEquals(Optional.empty(), result.get());
    }

    @Test
    void testLoadFanart() throws ExecutionException, InterruptedException {
        var media = mock(ShowDetails.class);
        var byteArray = mock(ByteArray.class);
        when(byteArray.getBytes()).thenReturn(new byte[0]);
        when(fxLib.load_fanart(eq(instance), isA(MediaItem.class))).thenReturn(byteArray);

        var future = imageService.loadFanart(media);
        var image = future.get();

        assertTrue(image.isPresent());
    }

    @Test
    void testLoadPoster() throws ExecutionException, InterruptedException {
        var media = mock(MovieOverview.class);
        var byteArray = mock(ByteArray.class);
        when(byteArray.getBytes()).thenReturn(new byte[0]);
        when(fxLib.load_poster(eq(instance), isA(MediaItem.class))).thenReturn(byteArray);

        var future = imageService.loadPoster(media);
        var image = future.get();

        assertTrue(image.isPresent());
    }

    @Test
    void testLoad() throws ExecutionException, InterruptedException {
        var url = "http://localhost/image.png";
        var byteArray = mock(ByteArray.class);
        when(byteArray.getBytes()).thenReturn(new byte[0]);
        when(fxLib.load_image(instance, url)).thenReturn(byteArray);

        var future = imageService.load(url);
        var image = future.get();

        assertNotNull(image);
    }

    private MovieDetails createMovie(Images images) {
        return MovieDetails.builder()
                .images(images)
                .build();
    }
}
