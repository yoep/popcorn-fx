package com.github.yoep.popcorn.ui.trakt;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.OAuth2AccessTokenWrapper;
import com.github.yoep.popcorn.backend.settings.models.TraktSettings;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.core.task.TaskExecutor;
import org.springframework.security.oauth2.client.OAuth2RestOperations;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class TraktServiceTest {
    @Mock
    private OAuth2RestOperations traktTemplate;
    @Mock
    private PopcornProperties properties;
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private WatchedService watchedService;
    @Mock
    private TaskExecutor taskExecutor;
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationEventPublisher eventPublisher;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private TraktSettings traktSettings;
    @InjectMocks
    private TraktService traktService;

    @BeforeEach
    void setUp() {
        lenient().when(settingsService.getSettings()).thenReturn(applicationSettings);
        lenient().when(applicationSettings.getTraktSettings()).thenReturn(traktSettings);
    }

    @Test
    void testIsAuthorized_whenInvokedAndTokenIsPresentInTheSettings_shouldReturnTrue() {
        var tokenWrapper = mock(OAuth2AccessTokenWrapper.class);
        when(traktSettings.getAccessToken()).thenReturn(Optional.of(tokenWrapper));

        var result = traktService.isAuthorized();

        assertTrue(result);
    }

    @Test
    void testIsAuthorized_whenInvokedAndTokenIsNotPresentInTheSettings_shouldReturnFalse() {
        when(traktSettings.getAccessToken()).thenReturn(Optional.empty());

        var result = traktService.isAuthorized();

        assertFalse(result);
    }

    @Test
    void testForget_whenInvoked_shouldSetTheTokenToNull() {
        traktService.forget();

        verify(traktSettings).setAccessToken(null);
    }
}
