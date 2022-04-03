package com.github.yoep.player.chromecast.services;

import com.github.yoep.player.chromecast.model.VideoMetadata;
import okhttp3.mockwebserver.MockResponse;
import okhttp3.mockwebserver.MockWebServer;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.web.reactive.function.client.WebClient;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.text.MessageFormat;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(MockitoExtension.class)
class MetaDataServiceTest {
    private static final MockWebServer MOCK_WEB_SERVER = new MockWebServer();

    @Spy
    private WebClient webClient = WebClient.builder().build();
    private MetaDataService service;

    @BeforeAll
    static void beforeAll() throws IOException {
        MOCK_WEB_SERVER.start();
    }

    @AfterAll
    static void afterAll() throws IOException {
        MOCK_WEB_SERVER.shutdown();
    }

    @BeforeEach
    void setUp() {
        service = new MetaDataService(webClient);
    }

    @Test
    void testResolveMetadata_whenContentTypeIsKnown_shouldReturnTheMediaType() throws URISyntaxException {
        var uri = buildUri();
        var expectedMediaType = MediaType.IMAGE_GIF_VALUE;
        var duration = 26000L;
        var expectedResult = VideoMetadata.builder()
                .contentType(expectedMediaType)
                .duration(duration)
                .build();
        MOCK_WEB_SERVER.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader(HttpHeaders.CONTENT_TYPE, expectedMediaType)
                .setHeader(HttpHeaders.CONTENT_LENGTH, duration));

        var result = service.resolveMetadata(uri);

        assertEquals(expectedResult, result);
    }

    @Test
    void testResolveMetadata_whenContentTypeIsNotPresent_shouldReturnOctetStream() throws URISyntaxException {
        var uri = buildUri();
        var duration = 86000L;
        var expectedResult = VideoMetadata.builder()
                .contentType(MediaType.APPLICATION_OCTET_STREAM_VALUE)
                .duration(duration)
                .build();
        MOCK_WEB_SERVER.enqueue(new MockResponse()
                .setResponseCode(200)
                .setHeader(HttpHeaders.CONTENT_LENGTH, duration));

        var result = service.resolveMetadata(uri);

        assertEquals(expectedResult, result);
    }

    private URI buildUri() throws URISyntaxException {
        var port = MOCK_WEB_SERVER.getPort();

        return new URI(MessageFormat.format("http://localhost:{0}", String.valueOf(port)));
    }
}