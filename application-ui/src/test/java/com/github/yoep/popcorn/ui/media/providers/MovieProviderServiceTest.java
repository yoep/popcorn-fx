package com.github.yoep.popcorn.ui.media.providers;

import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.config.properties.ProviderProperties;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.ui.settings.models.ServerSettings;
import com.github.yoep.popcorn.ui.view.models.Category;
import com.github.yoep.popcorn.ui.view.models.Genre;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.http.ResponseEntity;
import org.springframework.web.client.RestClientException;
import org.springframework.web.client.RestTemplate;

import java.net.URI;
import java.net.URISyntaxException;
import java.text.MessageFormat;
import java.util.Collections;
import java.util.Optional;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class MovieProviderServiceTest {
    @Mock
    private RestTemplate restTemplate;
    @Mock
    private ApplicationEventPublisher eventPublisher;
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

    private MovieProviderService movieProviderService;

    @BeforeEach
    void setUp() {
        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getServerSettings()).thenReturn(serverSettings);
        when(popcornConfig.getProvider(Category.MOVIES.getProviderName())).thenReturn(providerProperties);
    }

    @Nested
    class SupportsTest {
        @BeforeEach
        void setUp() {
            movieProviderService = new MovieProviderService(restTemplate, eventPublisher, popcornConfig, settingsService);
        }

        @Test
        void testSupports_whenCategoryIsFavorites_shouldReturnFalse() {
            var result = movieProviderService.supports(Category.FAVORITES);

            assertFalse(result);
        }

        @Test
        void testSupports_whenCategoryIsSeries_shouldReturnFalse() {
            var result = movieProviderService.supports(Category.SERIES);

            assertFalse(result);
        }

        @Test
        void testSupports_whenCategoryIsMovies_shouldReturnTrue() {
            var result = movieProviderService.supports(Category.MOVIES);

            assertTrue(result);
        }
    }

    @Nested
    class GetPageTest {
        @Test
        void testGetPage_whenCustomApiServerIsProvided_shouldUseCustomApiServerForPageRetrieval() throws URISyntaxException {
            var genreKey = "genreKey";
            var sortKey = "trending";
            var page = 1;
            var genre = new Genre(genreKey, "my genre text");
            var sort = new SortBy(sortKey, "my rending text");
            var apiUri = "http://my-api.com";
            var myApiServer = new URI(apiUri);
            var expectedUri = createApiUri(apiUri, genreKey, sortKey, page, "");
            when(serverSettings.getApiServer()).thenReturn(Optional.of(myApiServer));
            when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI("https://www.default-api.com")));
            when(restTemplate.getForEntity(isA(URI.class), eq(Movie[].class))).thenReturn(mock(ResponseEntity.class));
            movieProviderService = new MovieProviderService(restTemplate, eventPublisher, popcornConfig, settingsService);

            movieProviderService.getPage(genre, sort, page);

            verify(restTemplate).getForEntity(expectedUri, Movie[].class);
        }

        @Test
        void testGetPage_whenInvokedWithoutCustomApiServer_shouldUseDefaultApi() throws URISyntaxException {
            var genreKey = "genreKey";
            var sortKey = "trending";
            var page = 1;
            var defaultApi = "https://www.default-api.com";
            var genre = new Genre(genreKey, "my genre text");
            var sort = new SortBy(sortKey, "my rending text");
            var expectedUri = createApiUri(defaultApi, genreKey, sortKey, page, "");
            when(serverSettings.getApiServer()).thenReturn(Optional.empty());
            when(providerProperties.getUris()).thenReturn(Collections.singletonList(new URI(defaultApi)));
            when(restTemplate.getForEntity(isA(URI.class), eq(Movie[].class))).thenReturn(mock(ResponseEntity.class));
            movieProviderService = new MovieProviderService(restTemplate, eventPublisher, popcornConfig, settingsService);

            movieProviderService.getPage(genre, sort, page);

            verify(restTemplate).getForEntity(expectedUri, Movie[].class);
        }

        @Test
        void testGetPage_whenApiProviderFailed_shouldIterateToTheNextApi() throws URISyntaxException {
            var genreKey = "genreKey";
            var sortKey = "popularity";
            var page = 1;
            var apiServer1 = "https://www.default-api.com";
            var apiServer2 = "https://www.fallback-api.com";
            var genre = new Genre(genreKey, "my genre text");
            var sort = new SortBy(sortKey, "my rending text");
            var apiResponse = mock(ResponseEntity.class);
            var apiUri1 = createApiUri(apiServer1, genreKey, sortKey, page, "");
            var apiUri2 = createApiUri(apiServer2, genreKey, sortKey, page, "");
            when(serverSettings.getApiServer()).thenReturn(Optional.empty());
            when(providerProperties.getUris()).thenReturn(asList(new URI(apiServer1), new URI(apiServer2)));
            when(restTemplate.getForEntity(apiUri1, Movie[].class)).thenThrow(new RestClientException("my rest client failure"));
            when(restTemplate.getForEntity(apiUri2, Movie[].class)).thenReturn(apiResponse);
            movieProviderService = new MovieProviderService(restTemplate, eventPublisher, popcornConfig, settingsService);

            movieProviderService.getPage(genre, sort, page);

            verify(restTemplate).getForEntity(apiUri2, Movie[].class);
        }
    }

    private URI createApiUri(String apiUri, String genre, String sort, int page, String keywords) throws URISyntaxException {
        return new URI(MessageFormat.format("{0}/movies/{1}?sort={2}&order=-1&genre={3}&keywords={4}", apiUri, page, sort, genre, keywords));
    }
}
