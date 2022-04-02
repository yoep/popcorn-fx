package com.github.yoep.provider.anime.imdb;

import com.github.yoep.popcorn.backend.config.properties.ImdbProperties;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.provider.anime.media.models.Anime;
import mockwebserver3.MockResponse;
import mockwebserver3.MockWebServer;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.ClassPathResource;
import org.springframework.http.MediaType;
import org.springframework.web.reactive.function.client.WebClient;

import java.io.IOException;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.text.MessageFormat;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class TitleSearchScraperTest {
    private static MockWebServer mockServer = new MockWebServer();

    @Spy
    private WebClient webClient = WebClient.builder().build();
    @Mock
    private PopcornProperties popcornConfig;
    @Mock
    private ImdbProperties imdbProperties;
    @InjectMocks
    private TitleSearchScraper scraper;

    @BeforeAll
    static void beforeAll() throws IOException {
        mockServer.start();
    }

    @AfterAll
    static void afterAll() throws IOException {
        mockServer.shutdown();
    }

    @BeforeEach
    void setUp() {
        var port = mockServer.getPort();

        when(popcornConfig.getImdb()).thenReturn(imdbProperties);
        when(imdbProperties.getUrl()).thenReturn(URI.create(MessageFormat.format("http://localhost:{0}", String.valueOf(port))));
    }

    @Test
    void testRetrievePage_whenInvoked_shouldReturnExpectedResult() throws IOException {
        var resource = new ClassPathResource("title-search.html");
        var id1 = "tt2560140";
        var id2 = "tt11092142";
        var title1 = "Attack on Titan";
        var title2 = "Human Resources";
        var year1 = "2013";
        var year2 = "2022";
        var runtime1 = 24;
        var ratingPercentage1 = 91;
        var expectedResult = asList(Anime.builder()
                .nyaaId(id1)
                .imdbId(id1)
                .title(title1)
                .year(year1)
                .runtime(runtime1)
                .rating(Rating.builder()
                        .percentage(ratingPercentage1)
                        .build())
                .images(Images.builder().build())
                .build(), Anime.builder()
                .nyaaId(id2)
                .imdbId(id2)
                .title(title2)
                .year(year2)
                .runtime(30)
                .rating(Rating.builder()
                        .percentage(75)
                        .build())
                .images(Images.builder().build())
                .build());
        mockServer.enqueue(new MockResponse()
                .setResponseCode(200)
                .setBody(IOUtils.toString(resource.getInputStream(), StandardCharsets.UTF_8))
                .addHeader("Content-Type", MediaType.TEXT_HTML_VALUE));

        var result = scraper.retrievePage(new Genre("", ""), new SortBy("", ""), 0, null);

        assertEquals(2, result.getNumberOfElements());
        assertEquals(expectedResult, result.getContent());
    }
}