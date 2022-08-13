package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.ServerSettings;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.http.ResponseEntity;
import org.springframework.web.client.RestTemplate;

import java.net.URI;
import java.util.concurrent.ExecutionException;

import static java.util.Collections.singletonList;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class ShowProviderServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @Mock
    private PopcornProperties popcornConfig;
    @Mock
    private ProviderProperties providerProperties;
    @Mock
    private SettingsService settingsService;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private ServerSettings serverSettings;

    private ShowProviderService showProviderService;

    @BeforeEach
    void setUp() {
        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getServerSettings()).thenReturn(serverSettings);
        when(popcornConfig.getProvider(Category.SERIES.getProviderName())).thenReturn(providerProperties);

        showProviderService = new ShowProviderService(restTemplate, popcornConfig, settingsService);
    }

    @Nested
    class SupportsTest {
        @Test
        void testSupports_whenCategoryIsFavorites_shouldReturnFalse() {
            var result = showProviderService.supports(Category.FAVORITES);

            assertFalse(result);
        }

        @Test
        void testSupports_whenCategoryIsSeries_shouldReturnTrue() {
            var result = showProviderService.supports(Category.SERIES);

            assertTrue(result);
        }

        @Test
        void testSupports_whenCategoryIsMovies_shouldReturnFalse() {
            var result = showProviderService.supports(Category.MOVIES);

            assertFalse(result);
        }
    }

    @Test
    void testGetPage_whenNoKeywordsAreGiven_shouldRetrieveWithoutKeywords() throws ExecutionException, InterruptedException {
        var genre = new Genre(Genre.ALL_KEYWORD, "all");
        var sortBy = new SortBy("sortKey", "sortText");
        var uri = URI.create("http://localhost");
        var providerProperties = new ProviderProperties();
        var show = Show.builder().build();
        var response = new Show[]{show};
        var expectedResult = singletonList(show);
        providerProperties.setUris(singletonList(uri));
        when(restTemplate.getForEntity(isA(URI.class), eq(Show[].class))).thenReturn(ResponseEntity.ok(response));

        showProviderService.initializeUriProviders(serverSettings, providerProperties);
        var completableFuture = showProviderService.getPage(genre, sortBy, 1);
        var result = completableFuture.get();

        assertEquals(expectedResult, result.getContent());
    }

    @Test
    void testGetPage_whenKeywordsAreGiven_shouldRetrieveItemsWithKeywords() throws ExecutionException, InterruptedException {
        var genre = new Genre(Genre.ALL_KEYWORD, "all");
        var sortBy = new SortBy("sortKey", "sortText");
        var uri = URI.create("http://localhost");
        var providerProperties = new ProviderProperties();
        var keywords = "lorem";
        var show = Show.builder()
                .title(keywords)
                .build();
        var response = new Show[]{show};
        var expectedResult = singletonList(show);
        providerProperties.setUris(singletonList(uri));
        when(restTemplate.getForEntity(isA(URI.class), eq(Show[].class))).thenReturn(ResponseEntity.ok(response));

        showProviderService.initializeUriProviders(serverSettings, providerProperties);
        var completableFuture = showProviderService.getPage(genre, sortBy, 1, keywords);
        var result = completableFuture.get();

        assertEquals(expectedResult, result.getContent());
    }

    @Test
    void testRetrieveDetails_whenInvoked_shouldFetchMediaDetails() throws ExecutionException, InterruptedException {
        var show = Show.builder()
                .title("simple-show")
                .build();
        var expectedResult = Show.builder()
                .title("extended-show")
                .build();
        var providerProperties = new ProviderProperties();
        providerProperties.setUris(singletonList(URI.create("http://localhost")));
        when(restTemplate.getForEntity(isA(URI.class), eq(Show.class))).thenReturn(ResponseEntity.ok(expectedResult));

        showProviderService.initializeUriProviders(serverSettings, providerProperties);
        var completableFuture = showProviderService.retrieveDetails(show);
        var result = completableFuture.get();

        assertEquals(expectedResult, result);
    }
}
