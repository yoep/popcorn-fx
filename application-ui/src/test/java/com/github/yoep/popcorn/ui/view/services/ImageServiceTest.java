package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.media.providers.models.Images;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.web.client.RestTemplate;

import java.text.MessageFormat;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ImageServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @InjectMocks
    private ImageService imageService;

    @Test
    void testLoadFanart_whenMediaIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> imageService.loadPoster(null), "media cannot be null");
    }

    @Test
    void testLoadFanart_whenRemoteCallFails_shouldCallImageException() {
        var url = "http://my-fanart-url.com";
        var images = Images.builder()
                .fanart(url)
                .build();
        var media = createMovie(images);
        var response = mock(ResponseEntity.class);
        var statusCode = HttpStatus.BAD_REQUEST.value();
        var expectedMessage = MessageFormat.format("Failed to load image \"{0}\", expected status 2xx, but got {1} instead", url, statusCode);
        when(restTemplate.getForEntity(isA(String.class), eq(byte[].class))).thenReturn(response);
        when(response.getStatusCode()).thenReturn(HttpStatus.BAD_REQUEST);
        when(response.getStatusCodeValue()).thenReturn(statusCode);

        assertThrows(ImageException.class, () -> imageService.loadFanart(media), expectedMessage);
    }

    @Test
    void testLoadFanart_whenInvoked_shouldCallTheRemoteFanartUrl() {
        var url = "http://fanart-url.com";
        var images = Images.builder()
                .fanart(url)
                .build();
        var media = createMovie(images);
        var response = mock(ResponseEntity.class);
        when(restTemplate.getForEntity(isA(String.class), eq(byte[].class))).thenReturn(response);
        when(response.getStatusCode()).thenReturn(HttpStatus.OK);

        imageService.loadFanart(media);

        verify(restTemplate).getForEntity(url, byte[].class);
    }

    private Movie createMovie(Images images) {
        return Movie.builder()
                .images(images)
                .build();
    }
}
