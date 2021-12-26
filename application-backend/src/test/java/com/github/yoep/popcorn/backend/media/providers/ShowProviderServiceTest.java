package com.github.yoep.popcorn.backend.media.providers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.ServerSettings;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.web.client.RestTemplate;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class ShowProviderServiceTest {
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
    @Mock
    private LocaleText localeText;

    private ShowProviderService showProviderService;

    @BeforeEach
    void setUp() {
        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getServerSettings()).thenReturn(serverSettings);
        when(popcornConfig.getProvider(Category.SERIES.getProviderName())).thenReturn(providerProperties);

        showProviderService = new ShowProviderService(restTemplate, eventPublisher, popcornConfig, localeText, settingsService);
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
}
