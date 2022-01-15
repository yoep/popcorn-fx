package com.github.yoep.provider.anime.media;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.ServerSettings;
import org.apache.commons.io.IOUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.ClassPathResource;
import org.springframework.http.ResponseEntity;
import org.springframework.web.client.RestTemplate;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.nio.charset.StandardCharsets;
import java.text.MessageFormat;
import java.util.Collections;
import java.util.concurrent.ExecutionException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class AnimeProviderServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @Mock
    private PopcornProperties popcornConfig;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ServerSettings serverSettings;
    @Mock
    private ProviderProperties providerProperties;

    private AnimeProviderService service;

    @BeforeEach
    void setUp() {
        var applicationSettings = ApplicationSettings.builder()
                .serverSettings(serverSettings)
                .build();

        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(popcornConfig.getProvider(Category.ANIME.getProviderName())).thenReturn(providerProperties);

        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService);
    }

    @Test
    void testSupports_whenCategoryIsAnime_shouldReturnTrue() {
        var category = Category.ANIME;

        var result = service.supports(category);

        assertTrue(result);
    }

    @Test
    void testSupports_whenCategoryIsMovie_shouldReturnFalse() {
        var category = Category.MOVIES;

        var result = service.supports(category);

        assertFalse(result);
    }

    @Test
    void testGetPage_whenGenreIsAnimeMusicVideo_shouldRetrieveAnimeDataForTheGenre()
            throws URISyntaxException, IOException, ExecutionException, InterruptedException {
        var genre = new Genre("anime-music-video", "lorem");
        var sortBy = new SortBy("popular", "ipsum");
        var page = 1;
        var response = getResourceHtml("overview.html");
        var uri = "https://www.default-api.com";
        var expectedUri = new URI(MessageFormat.format("{0}?f={1}&c={2}&p={3}", uri, 2, "1_1", page));
        when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI(uri)));
        when(restTemplate.getForEntity(isA(URI.class), eq(String.class))).thenReturn(ResponseEntity.ok(response));
        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService);

        var completableFuture = service.getPage(genre, sortBy, page);

        verify(restTemplate).getForEntity(expectedUri, String.class);
        var result = completableFuture.get();
        assertEquals(2, result.getTotalElements());
    }

    @Test
    void testGetPage_whenGenreIsEnglishTranslated_shouldRetrieveAnimeDataForTheGenre()
            throws URISyntaxException, IOException, ExecutionException, InterruptedException {
        var genre = new Genre("english-translated", "lorem");
        var sortBy = new SortBy("popular", "ipsum");
        var page = 1;
        var response = getResourceHtml("overview.html");
        var uri = "https://www.default-api.com";
        var expectedUri = new URI(MessageFormat.format("{0}?f={1}&c={2}&p={3}", uri, 2, "1_2", page));
        when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI(uri)));
        when(restTemplate.getForEntity(isA(URI.class), eq(String.class))).thenReturn(ResponseEntity.ok(response));
        service = new AnimeProviderService(restTemplate, popcornConfig, settingsService);

        var completableFuture = service.getPage(genre, sortBy, page);

        verify(restTemplate).getForEntity(expectedUri, String.class);
        var result = completableFuture.get();
        assertEquals(2, result.getTotalElements());
    }

    private String getResourceHtml(String filename) throws IOException {
        var resource = new ClassPathResource(filename);

        return IOUtils.toString(resource.getInputStream(), StandardCharsets.UTF_8);
    }
}