package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.lib.ByteArray;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.web.client.RestTemplate;

import java.util.Optional;
import java.util.concurrent.ExecutionException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class ImageServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private ImageService imageService;

    @Test
    void testGetPosterHolder() {
        var byteArray = mock(ByteArray.class);
        when(byteArray.getBytes()).thenReturn(new byte[0]);
        when(fxLib.poster_holder(instance)).thenReturn(byteArray);

        var result = imageService.getPosterHolder();

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

    private MovieDetails createMovie(Images images) {
        return MovieDetails.builder()
                .images(images)
                .build();
    }
}
