package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.PlaybackSettings;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import com.github.yoep.popcorn.ui.messages.SettingsMessage;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.invocation.InvocationOnMock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
public class SettingsPlaybackComponentTest {
    @Spy
    private EventPublisher activityManager = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private ApplicationSettings applicationSettings;
    @Mock
    private PlaybackSettings playbackSettings;
    @InjectMocks
    private SettingsPlaybackComponent settingsPlaybackComponent;

    @BeforeEach
    void setUp() {
        when(settingsService.getSettings()).thenReturn(applicationSettings);
        when(applicationSettings.getPlaybackSettings()).thenReturn(playbackSettings);
    }

    @Test
    void testShowNotification_whenActivityIsInvokedInvoked_shouldCallLocaleTextWithSettingsSaved() {
        doAnswer(this::invokeSuccessNotification).when(activityManager).publishEvent(isA(SuccessNotificationEvent.class));

        settingsPlaybackComponent.onQualityChanged(PlaybackSettings.Quality.p720);

        verify(localeText).get(SettingsMessage.SETTINGS_SAVED);
    }

    @Test
    void testOnQualityChanged_whenInvoked_shouldShowNotification() {
        settingsPlaybackComponent.onQualityChanged(PlaybackSettings.Quality.p1080);

        verify(activityManager).publishEvent(isA(SuccessNotificationEvent.class));
    }

    @Test
    void testOnQualityChanged_whenInvoked_shouldUpdateQualitySetting() {
        var quality = PlaybackSettings.Quality.p1080;

        settingsPlaybackComponent.onQualityChanged(quality);

        verify(playbackSettings).setQuality(quality);
    }

    @Test
    void testOnFullscreenChanged_whenInvoked_shouldShowNotification() {
        settingsPlaybackComponent.onFullscreenChanged(false);

        verify(activityManager).publishEvent(isA(SuccessNotificationEvent.class));
    }

    @Test
    void testOnFullscreenChanged_whenInvoked_shouldUpdateTheFullscreenSetting() {
        settingsPlaybackComponent.onFullscreenChanged(true);

        verify(playbackSettings).setFullscreen(true);
    }

    @Test
    void testOnAutoPlayNextEpisodeChanged_whenInvoked_shouldShowNotification() {
        settingsPlaybackComponent.onAutoPlayNextEpisodeChanged(true);

        verify(activityManager).publishEvent(isA(SuccessNotificationEvent.class));
    }

    @Test
    void testOnAutoPlayNextEpisodeChanged_whenInvoked_shouldUpdateTheFullscreenSetting() {
        settingsPlaybackComponent.onAutoPlayNextEpisodeChanged(false);

        verify(playbackSettings).setAutoPlayNextEpisodeEnabled(false);
    }

    private Void invokeSuccessNotification(InvocationOnMock invocation) {
        var activity = (SuccessNotificationEvent) invocation.getArgument(0);
        activity.getText();
        return null;
    }
}
