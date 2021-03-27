package com.github.yoep.torrent.stream.services;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.Resource;
import org.springframework.http.MediaType;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class MediaServiceTest {
    @InjectMocks
    private MediaService mediaService;

    @Test
    void testContentType_whenVideoArgumentIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> mediaService.contentType(null), "video cannot be null");
    }

    @Test
    void testContentType_whenVideoIsMp4_shouldReturnVideoMp4MediaType() {
        var video = mock(Resource.class);
        var filename = "my-video.mp4";
        var expectedResult = MediaType.valueOf("video/mp4");
        when(video.getFilename()).thenReturn(filename);

        var result = mediaService.contentType(video);

        assertEquals(expectedResult, result);
    }

    @Test
    void testContentType_whenVideoIsMkv_shouldReturnVideoMatroskaMediaType() {
        var video = mock(Resource.class);
        var filename = "my-video.mkv";
        var expectedResult = MediaType.valueOf("video/x-matroska");
        when(video.getFilename()).thenReturn(filename);

        var result = mediaService.contentType(video);

        assertEquals(expectedResult, result);
    }

    @Test
    void testContentType_whenVideoIsUnknownExtension_shouldReturnOctetStreamMediaType() {
        var video = mock(Resource.class);
        var filename = "my-video.unknown";
        var expectedResult = MediaType.APPLICATION_OCTET_STREAM;
        when(video.getFilename()).thenReturn(filename);

        var result = mediaService.contentType(video);

        assertEquals(expectedResult, result);
    }
}
